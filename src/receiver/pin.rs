//! Embedded-hal pin based receiver

use embedded_hal::digital::InputPin;

use crate::{
    cmd::AnyCommand,
    receiver::{
        time::InfraMonotonic, DecoderBuilder, EdgeDriven, Error, MultiReceiver, PolledReceiver,
        PollDriven, Receiver, ReceiverSet, Timeout,
    },
    Protocol,
};

/// Receiver wrapper that reads its input from an embedded-hal `InputPin`
///
/// Wraps any pinless receiver core ([`Receiver`], [`MultiReceiver`] or
/// [`PolledReceiver`]) and reads the pin state on every event.
///
/// Note: IR receiver modules are typically active-low, so a low pin is
/// interpreted as "IR mark" (carrier detected).
pub struct PinReceiver<R, Pin> {
    recv: R,
    pin: Pin,
}

impl<R, Pin> PinReceiver<R, Pin> {
    /// Combine the receiver `recv` with `pin` as its input
    pub fn new(recv: R, pin: Pin) -> Self {
        Self { recv, pin }
    }

    /// Get a reference to the wrapped receiver
    pub fn receiver(&self) -> &R {
        &self.recv
    }

    /// Get a mut reference to the wrapped receiver
    pub fn receiver_mut(&mut self) -> &mut R {
        &mut self.recv
    }

    /// Get a reference to the Pin
    pub fn pin(&self) -> &Pin {
        &self.pin
    }

    /// Get a mut reference to the Pin
    pub fn pin_mut(&mut self) -> &mut Pin {
        &mut self.pin
    }

    /// Destroy the receiver and release the parts
    pub fn release(self) -> (R, Pin) {
        (self.recv, self.pin)
    }
}

impl<R, Pin> PinReceiver<R, Pin>
where
    R: EdgeDriven,
    Pin: InputPin,
{
    /// Update the receiver
    /// * `dt`: Duration since the previous edge
    pub fn event(&mut self, dt: R::Duration) -> Result<R::Output, Error<Pin::Error>> {
        let edge = self.pin.is_low().map_err(Error::Hal)?;
        self.recv.event_edge(dt, edge).map_err(Into::into)
    }

    /// Update the receiver with an edge at instant `t`
    ///
    /// The duration since the previous edge is tracked internally.
    pub fn event_instant(&mut self, t: R::Instant) -> Result<R::Output, Error<Pin::Error>> {
        let edge = self.pin.is_low().map_err(Error::Hal)?;
        self.recv.event_edge_instant(t, edge).map_err(Into::into)
    }
}

impl<R, Pin> PinReceiver<R, Pin>
where
    R: Timeout,
    Pin: InputPin,
{
    /// Notify the receiver that the input has been idle
    ///
    /// See [`Timeout`].
    pub fn timeout(&mut self) -> Result<R::Output, Error<Pin::Error>> {
        self.recv.timeout().map_err(Into::into)
    }
}

impl<R, Pin> PinReceiver<R, Pin>
where
    R: PollDriven,
    Pin: InputPin,
{
    /// Sample the pin and update the receiver
    ///
    /// To be called periodically at the receiver's sample frequency
    pub fn poll(&mut self) -> Result<R::Output, Error<Pin::Error>> {
        let edge = self.pin.is_low().map_err(Error::Hal)?;
        self.recv.poll_edge(edge).map_err(Into::into)
    }
}

impl<Proto, Pin, Cmd> PinReceiver<PolledReceiver<Proto, Cmd>, Pin>
where
    Proto: DecoderBuilder<u32>,
    Pin: InputPin,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    /// Create a periodic poll receiver with the given sample frequency (in Hz)
    /// and `pin` as input
    pub fn with_pin_poll(freq: u32, pin: Pin) -> Self {
        Self::new(PolledReceiver::new(freq), pin)
    }
}

impl<Proto, Pin, Mono, Cmd> PinReceiver<Receiver<Proto, Mono, Cmd>, Pin>
where
    Proto: DecoderBuilder<Mono>,
    Pin: InputPin,
    Mono: InfraMonotonic,
    Cmd: From<Proto::Cmd>,
{
    /// Create a `Receiver` with the given clock resolution (in Hz) and `pin` as input
    pub fn with_pin(freq: u32, pin: Pin) -> Self {
        Self::new(Receiver::new(freq), pin)
    }
}

#[cfg(feature = "fugit")]
impl<Proto, Pin, const HZ: u32, Cmd>
    PinReceiver<Receiver<Proto, fugit::TimerInstantU32<HZ>, Cmd>, Pin>
where
    Proto: DecoderBuilder<fugit::TimerInstantU32<HZ>>,
    Pin: InputPin,
    Cmd: From<Proto::Cmd>,
{
    /// Create a `Receiver` with fugit `TimerInstantU32<HZ>` as its clock and `pin` as input
    pub fn with_fugit(pin: Pin) -> Self {
        Self::new(Receiver::with_fugit(), pin)
    }
}

#[cfg(feature = "fugit")]
impl<Proto, Pin, const HZ: u32, Cmd>
    PinReceiver<Receiver<Proto, fugit::TimerInstantU64<HZ>, Cmd>, Pin>
where
    Proto: DecoderBuilder<fugit::TimerInstantU64<HZ>>,
    Pin: InputPin,
    Cmd: From<Proto::Cmd>,
{
    /// Create a `Receiver` with fugit `TimerInstantU64<HZ>` as its clock and `pin` as input
    pub fn with_fugit64(pin: Pin) -> Self {
        Self::new(Receiver::with_fugit64(), pin)
    }
}

impl<const N: usize, Receivers, Pin, Mono> PinReceiver<MultiReceiver<N, Receivers, Mono>, Pin>
where
    Receivers: ReceiverSet<N, Mono>,
    Pin: InputPin,
    Mono: InfraMonotonic,
{
    /// Create a `MultiReceiver` with the given clock resolution (in Hz) and `pin` as input
    pub fn with_pin_multi(freq: u32, pin: Pin) -> Self {
        Self::new(MultiReceiver::new(freq), pin)
    }

    /// Update the receiver and get an iterator over the decoded commands
    /// * `dt`: Duration since the previous edge
    pub fn event_iter(
        &mut self,
        dt: Mono::Duration,
    ) -> Result<impl Iterator<Item = AnyCommand>, Error<Pin::Error>> {
        Ok(self.event(dt)?.into_iter().flatten())
    }

    /// Update the receiver with an edge at instant `t` and get an iterator
    /// over the decoded commands
    pub fn event_instant_iter(
        &mut self,
        t: Mono::Instant,
    ) -> Result<impl Iterator<Item = AnyCommand>, Error<Pin::Error>> {
        Ok(self.event_instant(t)?.into_iter().flatten())
    }
}
