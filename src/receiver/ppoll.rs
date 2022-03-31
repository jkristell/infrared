use core::marker::PhantomData;
#[cfg(feature = "embedded")]
use embedded_hal::digital::v2::InputPin;

use crate::{
    receiver::{DecodingError, Error, NoPinInput, DecoderAdapter},
    Protocol,
};
use crate::receiver::ProtocolDecoder;

pub struct PeriodicPoll<
    Proto: DecoderAdapter<u32>,
    Input = NoPinInput,
    Cmd: From<<Proto as Protocol>::Cmd> = <Proto as Protocol>::Cmd,
> {
    /// Our internal clock
    clock: u32,
    /// Last seen edge
    edge: bool,
    /// Seen at
    last_edge: u32,
    /// The decoder
    decoder: Proto::Decoder,
    /// Input pin or NoPinInput
    input: Input,
    /// Command
    cmd: PhantomData<Cmd>,
}

impl<Proto, Input, Cmd> PeriodicPoll<Proto, Input, Cmd>
where
    Proto: DecoderAdapter<u32>,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn with_input(freq: u32, input: Input) -> Self {
        Self {
            decoder: Proto::decoder(freq),
            input,
            clock: 0,
            edge: false,
            last_edge: 0,
            cmd: PhantomData
        }
    }

    pub fn poll_base(&mut self, edge: bool) -> Result<Option<Cmd>, DecodingError> {
        self.clock = self.clock.wrapping_add(1);

        if edge == self.edge {
            return Ok(None);
        }

        let ds = self.clock.wrapping_sub(self.last_edge);
        self.edge = edge;
        self.last_edge = self.clock;

        self.decoder.combined(edge, ds).map(|cmd| cmd.map(Into::into))
    }
}

impl<Proto, Cmd> PeriodicPoll<Proto, NoPinInput, Cmd>
    where
        Proto: DecoderAdapter<u32>,
        Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn new(freq: u32) -> Self {
        Self::with_input(freq, NoPinInput)
    }

    pub fn poll(&mut self, edge: bool) -> Result<Option<Cmd>, DecodingError> {
        self.poll_base(edge)
    }
}



#[cfg(feature = "embedded-hal")]
impl<Proto, Pin, Cmd> PeriodicPoll<Proto, Pin, Cmd>
where
    Proto: DecoderAdapter<u32>,
    Pin: InputPin,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn with_pin(freq: u32, pin: Pin) -> PeriodicPoll<Proto, Pin, Cmd> {
        Self::with_input(freq, pin)
    }

    pub fn poll(&mut self) -> Result<Option<Cmd>, Error<Pin::Error>> {
        let edge = self.input.is_low().map_err(Error::Hal)?;

        self.poll_base(edge).map_err(Into::into)
    }
}
