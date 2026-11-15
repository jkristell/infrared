use core::marker::PhantomData;

use crate::{
    Level,
    receiver::{DecoderBuilder, DecodingError, PollDriven, ProtocolDecoder, Timeout},
    Protocol,
};

/// Periodic poll Receiver
///
/// The protocol decoder state machine for periodically sampled input.
/// Hardware independent, like [`Receiver`](crate::receiver::Receiver): the
/// sampled pin state is fed to [`poll`](PolledReceiver::poll) manually. Combine
/// it with [`PinReceiver`](crate::receiver::PinReceiver) to let it sample an
/// embedded-hal pin.
///
/// The receiver detects idle input by itself: when the input hasn't changed
/// for [`idle_timeout`](PolledReceiver::idle_timeout) samples, any partially
/// received frame is discarded.
pub struct PolledReceiver<
    Proto: DecoderBuilder<u32>,
    Cmd: From<<Proto as Protocol>::Cmd> = <Proto as Protocol>::Cmd,
> {
    /// Our internal clock
    clock: u32,
    /// Last seen edge
    level: Level,
    /// Seen at
    last_edge: u32,
    /// Number of samples without an edge after which the input is idle
    idle_samples: u32,
    /// The decoder
    decoder: Proto::Decoder,
    /// Command
    cmd: PhantomData<Cmd>,
}

impl<Proto, Cmd> PolledReceiver<Proto, Cmd>
where
    Proto: DecoderBuilder<u32>,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    /// Create a receiver polled at `sample_rate` Hz
    pub fn new(sample_rate: u32) -> Self {
        let decoder = Proto::build(sample_rate);
        Self {
            idle_samples: decoder.spans().idle_timeout(),
            decoder,
            clock: 0,
            level: Level::Space,
            last_edge: 0,
            cmd: PhantomData,
        }
    }

    /// Update the receiver with the current pin state
    ///
    /// To be called periodically at the sample frequency
    pub fn poll(&mut self, level: Level) -> Result<Option<Cmd>, DecodingError> {
        self.clock = self.clock.wrapping_add(1);

        if level == self.level {
            if self.clock.wrapping_sub(self.last_edge) == self.idle_samples {
                return self.timeout();
            }
            return Ok(None);
        }

        let ds = self.clock.wrapping_sub(self.last_edge);
        self.level = level;
        self.last_edge = self.clock;

        self.decoder
            .event_total(ds, level)
            .map(|cmd| cmd.map(Into::into))
    }

    /// Discard any partially received frame
    ///
    /// Called automatically by [`poll`](PolledReceiver::poll) once the input
    /// has been idle for [`idle_timeout`](PolledReceiver::idle_timeout) samples.
    pub fn timeout(&mut self) -> Result<Option<Cmd>, DecodingError> {
        self.decoder
            .timeout_total()
            .map(|cmd| cmd.map(Into::into))
    }

    /// Number of samples without an edge after which the input is considered idle
    pub fn idle_timeout(&self) -> u32 {
        self.idle_samples
    }
}

impl<Proto, Cmd> PollDriven for PolledReceiver<Proto, Cmd>
where
    Proto: DecoderBuilder<u32>,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    type Output = Option<Cmd>;

    fn poll_edge(&mut self, level: Level) -> Result<Self::Output, DecodingError> {
        self.poll(level)
    }
}

impl<Proto, Cmd> Timeout for PolledReceiver<Proto, Cmd>
where
    Proto: DecoderBuilder<u32>,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    type Output = Option<Cmd>;

    fn timeout(&mut self) -> Result<Self::Output, DecodingError> {
        PolledReceiver::timeout(self)
    }
}
