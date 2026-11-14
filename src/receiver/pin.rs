//! Embedded-hal pin based receiver

use embedded_hal::digital::InputPin;

use crate::receiver::{
    time::InfraMonotonic, DecoderBuilder, EdgeReceiver, Error, MultiReceiver, Receiver,
    ReceiverWrapper,
};
use crate::cmd::AnyCommand;

/// Receiver wrapper that reads its input from an embedded-hal `InputPin`
///
/// Wraps any [`EdgeReceiver`] (a [`Receiver`] or a [`MultiReceiver`]) and
/// reads the edge state from the pin on every event.
pub struct PinReceiver<R, Pin> {
    recv: R,
    pin: Pin,
}

impl<R, Pin> PinReceiver<R, Pin>
where
    R: EdgeReceiver,
    Pin: InputPin,
{
    /// Combine the receiver `recv` with `pin` as its input
    pub fn new(recv: R, pin: Pin) -> Self {
        Self { recv, pin }
    }

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
impl<Proto, Pin, const HZ: u32, Cmd> PinReceiver<Receiver<Proto, fugit::TimerInstantU32<HZ>, Cmd>, Pin>
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
impl<Proto, Pin, const HZ: u32, Cmd> PinReceiver<Receiver<Proto, fugit::TimerInstantU64<HZ>, Cmd>, Pin>
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
    Receivers: ReceiverWrapper<N, Mono>,
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
