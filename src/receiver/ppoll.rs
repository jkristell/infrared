use embedded_hal::digital::v2::InputPin;
use crate::{Protocol, Receiver};
use crate::receiver::{DecoderStateMachine, DefaultInput, Error, Event, PinInput};

pub struct PeriodicPoll<
    Proto: DecoderStateMachine<u32>,
    In,
    Cmd: From<<Proto as Protocol>::Cmd> = <Proto as Protocol>::Cmd,
>
{
    receiver: Receiver<Proto, Event, In, u32, Cmd>,

    clock: u32,
    /// Last seen edge
    edge: bool,
    /// Seen at
    last_edge: u32,
}

impl<Proto, Pin, Cmd> PeriodicPoll<Proto, PinInput<Pin>, Cmd>
where
    Proto: DecoderStateMachine<u32>,
    Pin: InputPin,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn poll(&mut self) -> Result<Option<Cmd>, Error<Pin::Error>> {
        let edge = self.receiver.pin().is_low().map_err(Error::Hal)?;

        self.clock = self.clock.wrapping_add(1);

        if edge == self.edge {
            return Ok(None);
        }

        let ds = self.clock.wrapping_sub(self.last_edge);
        self.edge = edge;
        self.last_edge = self.clock;

        Ok(self.receiver.generic_event(ds, edge)?.map(Into::into))
    }

}

