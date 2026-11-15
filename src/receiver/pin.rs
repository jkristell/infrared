//! Embedded-hal pin based receiver

use embedded_hal::digital::InputPin;

use crate::{
    Level,
    protocol::Message,
    receiver::{
        time::TimeBase, EdgeDriven, Error, MultiReceiver, PollDriven, ReceiverSet, Timeout,
    },
};

/// Receiver wrapper that reads its input from an embedded-hal `InputPin`
///
/// Wraps any pinless receiver core ([`Receiver`](crate::Receiver), [`MultiReceiver`] or
/// [`PolledReceiver`](crate::PolledReceiver)) and reads the pin state on every event.
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
        let level = self.pin.is_low().map(Level::from_active_low).map_err(Error::Pin)?;
        self.recv.event_edge(dt, level).map_err(Into::into)
    }

    /// Update the receiver with an edge at instant `t`
    ///
    /// The duration since the previous edge is tracked internally.
    pub fn event_at(&mut self, t: R::Instant) -> Result<R::Output, Error<Pin::Error>> {
        let level = self.pin.is_low().map(Level::from_active_low).map_err(Error::Pin)?;
        self.recv.event_edge_at(t, level).map_err(Into::into)
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
        let level = self.pin.is_low().map(Level::from_active_low).map_err(Error::Pin)?;
        self.recv.poll_edge(level).map_err(Into::into)
    }
}

impl<const N: usize, Receivers, Pin, Time> PinReceiver<MultiReceiver<N, Receivers, Time>, Pin>
where
    Receivers: ReceiverSet<N, Time>,
    Pin: InputPin,
    Time: TimeBase,
{
    /// Update the receiver and get an iterator over the decoded commands
    /// * `dt`: Duration since the previous edge
    pub fn event_iter(
        &mut self,
        dt: Time::Duration,
    ) -> Result<impl Iterator<Item = Message>, Error<Pin::Error>> {
        Ok(self.event(dt)?.into_iter().flatten())
    }

    /// Update the receiver with an edge at instant `t` and get an iterator
    /// over the decoded commands
    pub fn event_at_iter(
        &mut self,
        t: Time::Instant,
    ) -> Result<impl Iterator<Item = Message>, Error<Pin::Error>> {
        Ok(self.event_at(t)?.into_iter().flatten())
    }
}
