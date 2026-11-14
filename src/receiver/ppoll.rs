use core::marker::PhantomData;

#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::InputPin;

#[cfg(feature = "embedded-hal")]
use crate::receiver::PinReceiver;
use crate::{
    receiver::{DecoderBuilder, DecodingError, PollReceiver, ProtocolDecoder},
    Protocol,
};

/// Periodic poll Receiver
///
/// The protocol decoder state machine for periodically sampled input.
/// Hardware independent, like [`Receiver`](crate::receiver::Receiver): the
/// sampled pin state is fed to [`poll`](PeriodicPoll::poll) manually. Combine
/// it with [`PinReceiver`](crate::receiver::PinReceiver) to let it sample an
/// embedded-hal pin.
pub struct PeriodicPoll<
    Proto: DecoderBuilder<u32>,
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
    /// Command
    cmd: PhantomData<Cmd>,
}

impl<Proto, Cmd> PeriodicPoll<Proto, Cmd>
where
    Proto: DecoderBuilder<u32>,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    /// Create a new PeriodicPoll with the given sample frequency (in Hz)
    pub fn new(freq: u32) -> Self {
        Self {
            decoder: Proto::build(freq),
            clock: 0,
            edge: false,
            last_edge: 0,
            cmd: PhantomData,
        }
    }

    /// Update the receiver with the current pin state
    ///
    /// To be called periodically at the sample frequency
    pub fn poll(&mut self, edge: bool) -> Result<Option<Cmd>, DecodingError> {
        self.clock = self.clock.wrapping_add(1);

        if edge == self.edge {
            return Ok(None);
        }

        let ds = self.clock.wrapping_sub(self.last_edge);
        self.edge = edge;
        self.last_edge = self.clock;

        self.decoder
            .event_total(edge, ds)
            .map(|cmd| cmd.map(Into::into))
    }

    #[cfg(feature = "embedded-hal")]
    /// Create a periodic poll receiver with `pin` as input
    pub fn with_pin<Pin: InputPin>(freq: u32, pin: Pin) -> PinReceiver<Self, Pin> {
        PinReceiver::new(Self::new(freq), pin)
    }
}

impl<Proto, Cmd> PollReceiver for PeriodicPoll<Proto, Cmd>
where
    Proto: DecoderBuilder<u32>,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    type Output = Option<Cmd>;

    fn poll_edge(&mut self, edge: bool) -> Result<Self::Output, DecodingError> {
        self.poll(edge)
    }
}
