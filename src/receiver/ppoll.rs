#[cfg(feature = "embedded")]
use embedded_hal::digital::v2::InputPin;

use crate::{
    receiver::{DecodingError, Error, NoPinInput, ProtocolDecoderAdaptor},
    Protocol, Receiver,
};

pub struct PeriodicPoll<
    Proto: ProtocolDecoderAdaptor<u32>,
    Input = NoPinInput,
    Cmd: From<<Proto as Protocol>::Cmd> = <Proto as Protocol>::Cmd,
> {
    /// Our internal clock
    clock: u32,
    /// Last seen edge
    edge: bool,
    /// Seen at
    last_edge: u32,
    /// Our receiver state machine
    receiver: Receiver<Proto, Input, u32, Cmd>,
}

impl<Proto, Input, Cmd> PeriodicPoll<Proto, Input, Cmd>
where
    Proto: ProtocolDecoderAdaptor<u32>,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn new(resolution: u32, input: Input) -> Self {
        Self {
            receiver: Receiver::<Proto, Input, u32, Cmd>::with_input(resolution, input),
            clock: 0,
            edge: false,
            last_edge: 0,
        }
    }

    pub fn poll_edge(&mut self, edge: bool) -> Result<Option<Cmd>, DecodingError> {
        self.clock = self.clock.wrapping_add(1);

        if edge == self.edge {
            return Ok(None);
        }

        let ds = self.clock.wrapping_sub(self.last_edge);
        self.edge = edge;
        self.last_edge = self.clock;

        Ok(self.receiver.event_edge(ds, edge)?.map(Into::into))
    }
}

#[cfg(feature = "embedded-hal")]
impl<Proto, Pin, Cmd> PeriodicPoll<Proto, Pin, Cmd>
where
    Proto: ProtocolDecoderAdaptor<u32>,
    Pin: InputPin,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn with_pin(resolution: u32, pin: Pin) -> PeriodicPoll<Proto, Pin, Cmd> {
        Self::new(resolution, pin)
    }

    pub fn poll(&mut self) -> Result<Option<Cmd>, Error<Pin::Error>> {
        let edge = self.receiver.pin_mut().is_low().map_err(Error::Hal)?;

        self.poll_edge(edge).map_err(Into::into)
    }
}
