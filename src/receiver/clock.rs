//! Clock support
//!
//! A [`Clock`] lets the receiver timestamp edges itself, so the application
//! only has to report *that* an edge occurred, and lets the receiver decide by
//! itself when the input has gone idle.

use core::marker::PhantomData;

#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::InputPin;

#[cfg(feature = "embedded-hal")]
use crate::receiver::{Error, PinReceiver};
use crate::{
    cmd::AnyCommand,
    receiver::{
        DecoderBuilder, DecodingError, EdgeDriven, MultiReceiver, PollDriven, Receiver,
        ReceiverSet, Timeout, time::InfraMonotonic,
    },
};

type InstantOf<C> = <<C as Clock>::Mono as InfraMonotonic>::Instant;
type DurationOf<C> = <<C as Clock>::Mono as InfraMonotonic>::Duration;

/// A source of the current time
pub trait Clock {
    /// The time representation of the clock
    type Mono: InfraMonotonic;

    /// Resolution of the clock in Hz
    fn resolution(&self) -> u32;

    /// The current time
    fn now(&mut self) -> InstantOf<Self>;
}

/// [`Clock`] backed by a closure
///
/// Adapts any time source, such as a free running hardware timer, an RTIC
/// monotonic or a fugit instant, to a [`Clock`]:
///
/// ```
/// use infrared::receiver::{Clock, FnClock};
///
/// let mut ticks = 0u32;
/// // A 1 MHz counter
/// let mut clock: FnClock<_, u32> = FnClock::new(1_000_000, move || {
///     ticks += 10;
///     ticks
/// });
/// assert_eq!(clock.now(), 10);
/// ```
pub struct FnClock<F, Mono = u32> {
    resolution: u32,
    now: F,
    mono: PhantomData<Mono>,
}

impl<F, Mono> FnClock<F, Mono>
where
    Mono: InfraMonotonic,
    F: FnMut() -> Mono::Instant,
{
    /// Create a clock with `resolution` Hz that reads the time from `now`
    pub fn new(resolution: u32, now: F) -> Self {
        Self {
            resolution,
            now,
            mono: PhantomData,
        }
    }
}

impl<F, Mono> Clock for FnClock<F, Mono>
where
    Mono: InfraMonotonic,
    F: FnMut() -> Mono::Instant,
{
    type Mono = Mono;

    fn resolution(&self) -> u32 {
        self.resolution
    }

    fn now(&mut self) -> Mono::Instant {
        (self.now)()
    }
}

/// [`Clock`] backed by `std::time::Instant`, with microsecond resolution
#[cfg(feature = "std")]
pub struct StdClock {
    start: std::time::Instant,
}

#[cfg(feature = "std")]
impl StdClock {
    /// Create a clock that starts at zero now
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

#[cfg(feature = "std")]
impl Default for StdClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl Clock for StdClock {
    type Mono = u64;

    fn resolution(&self) -> u32 {
        1_000_000
    }

    fn now(&mut self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }
}

/// Receiver with its own [`Clock`]
///
/// Wraps an edge driven receiver core ([`Receiver`] or [`MultiReceiver`]) and
/// timestamps the edges using the clock. Since it knows the time of the last
/// edge it can also detect idle input by itself.
///
/// * [`event`](ClockedReceiver::event): report an edge, e.g. from the pin interrupt.
/// * [`timeout`](ClockedReceiver::timeout): call periodically, or when a
///   timer set to [`idle_timeout`](ClockedReceiver::idle_timeout) after the
///   last edge fires. Only acts when the input really has been idle long enough.
/// * [`poll`](ClockedReceiver::poll): report the current input level. Acts
///   as `event` if the level changed and as `timeout` otherwise, so the same
///   call can be used from both the pin interrupt and a periodic timer.
///
/// ```
/// use infrared::{protocol::Nec, receiver::{ClockedReceiver, FnClock}};
///
/// let clock: FnClock<_, u32> = FnClock::new(1_000_000, || 0);
/// let mut receiver = ClockedReceiver::<infrared::Receiver<Nec>, _>::with_clock(clock);
///
/// // Pin interrupt
/// let edge = true;
/// if let Ok(Some(cmd)) = receiver.event(edge) {
///     println!("{} {}", cmd.addr, cmd.cmd);
/// }
///
/// // Periodic timer
/// let _ = receiver.timeout();
/// ```
pub struct ClockedReceiver<R: EdgeDriven, C> {
    recv: R,
    clock: C,
    /// Time of the last edge
    last_edge: R::Instant,
    /// Level after the last edge
    level: bool,
    /// The input has been idle since the last timeout
    idle: bool,
    idle_timeout: R::Duration,
}

impl<R, C> ClockedReceiver<R, C>
where
    C: Clock,
    R: EdgeDriven<Instant = InstantOf<C>, Duration = DurationOf<C>>,
{
    /// Combine the receiver `recv` with `clock`
    pub fn new(recv: R, mut clock: C) -> Self {
        Self {
            last_edge: clock.now(),
            idle_timeout: recv.idle_timeout(),
            recv,
            clock,
            level: false,
            idle: true,
        }
    }

    /// Override the receiver's default idle timeout
    pub fn with_idle_timeout(mut self, idle_timeout: R::Duration) -> Self {
        self.idle_timeout = idle_timeout;
        self
    }

    /// Time without edges after which the input is considered idle
    pub fn idle_timeout(&self) -> R::Duration {
        self.idle_timeout
    }

    /// Time of the last edge
    pub fn last_edge(&self) -> R::Instant {
        self.last_edge
    }

    /// Report an edge that occurred now
    /// * `edge`: true = start of an IR mark (the pin reading low on an active-low receiver module), false = start of a space
    pub fn event(&mut self, edge: bool) -> Result<R::Output, DecodingError> {
        let now = self.clock.now();
        let dt = C::Mono::checked_sub(now, self.last_edge).unwrap_or(C::Mono::ZERO_DURATION);
        self.last_edge = now;
        self.level = edge;
        self.idle = false;
        self.recv.event_edge(dt, edge)
    }

    /// Get a reference to the wrapped receiver
    pub fn receiver(&self) -> &R {
        &self.recv
    }

    /// Get a mut reference to the wrapped receiver
    pub fn receiver_mut(&mut self) -> &mut R {
        &mut self.recv
    }

    /// Get a mut reference to the clock
    pub fn clock_mut(&mut self) -> &mut C {
        &mut self.clock
    }

    /// Destroy the receiver and release the parts
    pub fn release(self) -> (R, C) {
        (self.recv, self.clock)
    }
}

impl<R, C> ClockedReceiver<R, C>
where
    C: Clock,
    R: EdgeDriven<Instant = InstantOf<C>, Duration = DurationOf<C>>
        + Timeout<Output = <R as EdgeDriven>::Output>,
    <R as EdgeDriven>::Output: Default,
{
    /// Check whether the input has gone idle
    ///
    /// Discards any partially received frame if no edge has been seen for
    /// [`idle_timeout`](ClockedReceiver::idle_timeout). Does nothing otherwise,
    /// so it is safe to call at any time and as often as convenient.
    pub fn timeout(&mut self) -> Result<<R as EdgeDriven>::Output, DecodingError> {
        if self.idle {
            return Ok(Default::default());
        }

        let now = self.clock.now();
        match C::Mono::checked_sub(now, self.last_edge) {
            Some(elapsed) if elapsed >= self.idle_timeout => {
                self.idle = true;
                self.recv.timeout()
            }
            _ => Ok(Default::default()),
        }
    }

    /// Update the receiver with the current input level
    ///
    /// Handled as an [`event`](ClockedReceiver::event) if the level differs from
    /// the previous one, and as a [`timeout`](ClockedReceiver::timeout) check
    /// otherwise.
    pub fn poll(&mut self, level: bool) -> Result<<R as EdgeDriven>::Output, DecodingError> {
        if level != self.level {
            self.event(level)
        } else {
            self.timeout()
        }
    }
}

impl<R, C> Timeout for ClockedReceiver<R, C>
where
    C: Clock,
    R: EdgeDriven<Instant = InstantOf<C>, Duration = DurationOf<C>>
        + Timeout<Output = <R as EdgeDriven>::Output>,
    <R as EdgeDriven>::Output: Default,
{
    type Output = <R as EdgeDriven>::Output;

    fn timeout(&mut self) -> Result<Self::Output, DecodingError> {
        ClockedReceiver::timeout(self)
    }
}

impl<R, C> PollDriven for ClockedReceiver<R, C>
where
    C: Clock,
    R: EdgeDriven<Instant = InstantOf<C>, Duration = DurationOf<C>>
        + Timeout<Output = <R as EdgeDriven>::Output>,
    <R as EdgeDriven>::Output: Default,
{
    type Output = <R as EdgeDriven>::Output;

    fn poll_edge(&mut self, edge: bool) -> Result<Self::Output, DecodingError> {
        self.poll(edge)
    }
}

impl<Proto, C, Cmd> ClockedReceiver<Receiver<Proto, C::Mono, Cmd>, C>
where
    C: Clock,
    Proto: DecoderBuilder<C::Mono>,
    Cmd: From<Proto::Cmd>,
{
    /// Create a [`Receiver`] that uses `clock`
    pub fn with_clock(clock: C) -> Self {
        Self::new(Receiver::new(clock.resolution()), clock)
    }

    #[cfg(feature = "embedded-hal")]
    /// Create a [`Receiver`] that uses `clock` and `pin` as input
    pub fn with_clock_and_pin<Pin: InputPin>(clock: C, pin: Pin) -> PinReceiver<Self, Pin> {
        PinReceiver::new(Self::with_clock(clock), pin)
    }
}

impl<const N: usize, Receivers, C> ClockedReceiver<MultiReceiver<N, Receivers, C::Mono>, C>
where
    C: Clock,
    Receivers: ReceiverSet<N, C::Mono>,
{
    /// Create a [`MultiReceiver`] that uses `clock`
    pub fn with_clock_multi(clock: C) -> Self {
        Self::new(MultiReceiver::new(clock.resolution()), clock)
    }

    /// Report an edge that occurred now and get an iterator over the decoded commands
    pub fn event_iter(&mut self, edge: bool) -> impl Iterator<Item = AnyCommand> {
        self.event(edge).into_iter().flatten().flatten()
    }
}

#[cfg(feature = "embedded-hal")]
impl<R, C, Pin> PinReceiver<ClockedReceiver<R, C>, Pin>
where
    C: Clock,
    R: EdgeDriven<Instant = InstantOf<C>, Duration = DurationOf<C>>,
    Pin: InputPin,
{
    /// Read the pin and report an edge that occurred now
    ///
    /// To be called from the pin interrupt.
    pub fn event(&mut self) -> Result<R::Output, Error<Pin::Error>> {
        let edge = self.pin_mut().is_low().map_err(Error::Hal)?;
        self.receiver_mut().event(edge).map_err(Into::into)
    }
}
