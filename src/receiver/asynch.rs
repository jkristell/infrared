//! Async receiver

use embassy_futures::select::{select, Either};
use embedded_hal_async::{delay::DelayNs, digital::Wait};

use crate::{
    cmd::AnyCommand,
    receiver::{
        time::InfraMonotonic, Clock, ClockedReceiver, DecoderBuilder, EdgeDriven, Error,
        MultiReceiver, Receiver, ReceiverSet, Timeout,
    },
};

type InstantOf<C> = <<C as Clock>::Mono as InfraMonotonic>::Instant;
type DurationOf<C> = <<C as Clock>::Mono as InfraMonotonic>::Duration;

/// Async receiver
///
/// Waits for edges on an embedded-hal-async [`Wait`] pin, timestamps them
/// with a [`Clock`] and uses a [`DelayNs`] to detect idle input, so that
/// interrupted frames are discarded without any help from the application.
///
/// Works with any async HAL. With the `embassy` feature,
/// [`AsyncReceiver::embassy`] sets it up with embassy-time.
///
/// ```ignore
/// let mut receiver = infrared::receiver().nec().build_embassy(pin);
///
/// loop {
///     let cmd = receiver.receive().await?;
///     defmt::info!("{} {}", cmd.addr, cmd.cmd);
/// }
/// ```
///
/// Like [`PinReceiver`](crate::receiver::PinReceiver), the pin is assumed to
/// be connected to an active-low receiver module: low = IR mark.
pub struct AsyncReceiver<R: EdgeDriven, C, Pin, D> {
    recv: ClockedReceiver<R, C>,
    pin: Pin,
    delay: D,
    /// Current input level, true = IR mark
    level: bool,
}

impl<R, C, Pin, D> AsyncReceiver<R, C, Pin, D>
where
    C: Clock,
    R: EdgeDriven<Instant = InstantOf<C>, Duration = DurationOf<C>>
        + Timeout<Output = <R as EdgeDriven>::Output>,
    Pin: Wait,
    D: DelayNs,
{
    /// Create an async receiver from a receiver core, a clock, the input pin
    /// and a delay used for the idle timeout
    pub fn new(recv: R, clock: C, pin: Pin, delay: D) -> Self {
        Self {
            recv: ClockedReceiver::new(recv, clock),
            pin,
            delay,
            level: false,
        }
    }

    /// Override the receiver's default idle timeout
    pub fn with_idle_timeout(mut self, idle_timeout: R::Duration) -> Self {
        self.recv = self.recv.with_idle_timeout(idle_timeout);
        self
    }

    /// Wait for the next edge, or for the input to go idle, and update the receiver
    ///
    /// Most applications use `receive()` instead. This lower level method
    /// also returns decoding errors.
    pub async fn next(&mut self) -> Result<<R as EdgeDriven>::Output, Error<Pin::Error>> {
        let to_mark = !self.level;
        let pin = &mut self.pin;
        let edge = async {
            if to_mark {
                pin.wait_for_low().await
            } else {
                pin.wait_for_high().await
            }
        };

        let edge = if self.recv.is_idle() {
            Some(edge.await)
        } else {
            let us = C::Mono::to_micros(self.recv.resolution(), self.recv.idle_timeout());
            let us = u32::try_from(us).unwrap_or(u32::MAX);
            match select(edge, self.delay.delay_us(us)).await {
                Either::First(res) => Some(res),
                Either::Second(()) => None,
            }
        };

        match edge {
            Some(res) => {
                res.map_err(Error::Hal)?;
                self.level = to_mark;
                self.recv.event(to_mark).map_err(Into::into)
            }
            None => self.recv.force_timeout().map_err(Into::into),
        }
    }

    /// Get a reference to the wrapped receiver
    pub fn receiver(&self) -> &ClockedReceiver<R, C> {
        &self.recv
    }

    /// Get a reference to the pin
    pub fn pin(&self) -> &Pin {
        &self.pin
    }

    /// Get a mut reference to the pin
    pub fn pin_mut(&mut self) -> &mut Pin {
        &mut self.pin
    }

    /// Destroy the receiver and release the parts
    pub fn release(self) -> (R, C, Pin, D) {
        let (recv, clock) = self.recv.release();
        (recv, clock, self.pin, self.delay)
    }
}

impl<Proto, C, Pin, D, Cmd> AsyncReceiver<Receiver<Proto, C::Mono, Cmd>, C, Pin, D>
where
    C: Clock,
    Proto: DecoderBuilder<C::Mono>,
    Cmd: From<Proto::Cmd>,
    Pin: Wait,
    D: DelayNs,
{
    /// Create an async [`Receiver`]
    pub fn with_clock(clock: C, pin: Pin, delay: D) -> Self {
        Self::new(Receiver::new(clock.resolution()), clock, pin, delay)
    }

    /// Wait for the next command
    ///
    /// Decoding errors are skipped. Only pin errors are returned.
    pub async fn receive(&mut self) -> Result<Cmd, Pin::Error> {
        loop {
            match self.next().await {
                Ok(Some(cmd)) => return Ok(cmd),
                Err(Error::Hal(err)) => return Err(err),
                _ => {}
            }
        }
    }
}

impl<const N: usize, Receivers, C, Pin, D>
    AsyncReceiver<MultiReceiver<N, Receivers, C::Mono>, C, Pin, D>
where
    C: Clock,
    Receivers: ReceiverSet<N, C::Mono>,
    Pin: Wait,
    D: DelayNs,
{
    /// Create an async [`MultiReceiver`]
    pub fn with_clock_multi(clock: C, pin: Pin, delay: D) -> Self {
        Self::new(MultiReceiver::new(clock.resolution()), clock, pin, delay)
    }

    /// Wait for the next command from any of the protocols
    ///
    /// Decoding errors are skipped. Only pin errors are returned. In the
    /// unlikely case that several protocols decode the same frame, the first
    /// one in the protocol list is returned.
    pub async fn receive(&mut self) -> Result<AnyCommand, Pin::Error> {
        loop {
            match self.next().await {
                Ok(cmds) => {
                    if let Some(cmd) = cmds.into_iter().flatten().next() {
                        return Ok(cmd);
                    }
                }
                Err(Error::Hal(err)) => return Err(err),
                Err(_) => {}
            }
        }
    }
}

/// [`Clock`] backed by embassy-time, with microsecond resolution
#[cfg(feature = "embassy")]
pub struct EmbassyClock;

#[cfg(feature = "embassy")]
impl Clock for EmbassyClock {
    type Mono = u64;

    fn resolution(&self) -> u32 {
        1_000_000
    }

    fn now(&mut self) -> u64 {
        embassy_time::Instant::now().as_micros()
    }
}

#[cfg(feature = "embassy")]
impl<Proto, Pin, Cmd> AsyncReceiver<Receiver<Proto, u64, Cmd>, EmbassyClock, Pin, embassy_time::Delay>
where
    Proto: DecoderBuilder<u64>,
    Cmd: From<Proto::Cmd>,
    Pin: Wait,
{
    /// Create an async [`Receiver`] using embassy-time
    pub fn embassy(pin: Pin) -> Self {
        Self::with_clock(EmbassyClock, pin, embassy_time::Delay)
    }
}
