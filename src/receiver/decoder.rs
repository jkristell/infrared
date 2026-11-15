use core::fmt::Debug;

use crate::{
    receiver::{
        time::{TimeBase, PulseSpans},
        DecodingError,
    },
    Level, Protocol,
};

/// Used to create a Decoder for a protocol
///
/// Handles the creation of the pulse spans for the protocol
pub trait DecoderBuilder<Time: TimeBase>: Protocol + Sized {
    /// Type of the decoder
    type Decoder: ProtocolDecoder<Self, Time>;

    /// Create the decoder for timestamps counting at `resolution` Hz
    fn build(resolution: u32) -> Self::Decoder;
}

/// Protocol decode state machine
pub trait ProtocolDecoder<Proto, Time>
where
    Proto: Protocol + ?Sized,
    Time: TimeBase,
{
    /// Notify the state machine of an edge
    /// * `dt`: Duration since the previous edge
    /// * `level`: The level the signal changed to
    fn event(&mut self, dt: Time::Duration, level: Level) -> DecoderState;

    /// Get the command
    /// Returns the data if DecoderState == Done, otherwise None
    fn command(&self) -> Option<Proto::Cmd>;

    /// Reset the decoder
    fn reset(&mut self);

    /// Notify the state machine that the input has been idle for longer than
    /// any valid symbol of the protocol
    ///
    /// The default implementation discards a partially received frame.
    /// Protocols whose frames can only be terminated by silence override this
    /// to finish the frame and return [`DecoderState::Done`].
    fn timeout(&mut self) -> DecoderState {
        self.reset();
        DecoderState::Idle
    }

    /// Get the time spans
    fn spans(&self) -> &PulseSpans<Time>;

    #[doc(hidden)]
    fn event_total(
        &mut self,
        dt: Time::Duration,
        level: Level,
    ) -> Result<Option<Proto::Cmd>, DecodingError> {
        let state = self.event(dt, level);
        self.finish(state)
    }

    #[doc(hidden)]
    fn timeout_total(&mut self) -> Result<Option<Proto::Cmd>, DecodingError> {
        let state = self.timeout();
        self.finish(state)
    }

    #[doc(hidden)]
    fn finish(&mut self, state: DecoderState) -> Result<Option<Proto::Cmd>, DecodingError> {
        match state {
            DecoderState::Idle | DecoderState::Receiving => Ok(None),
            DecoderState::Done => {
                let cmd = self.command();
                self.reset();
                Ok(cmd)
            }
            DecoderState::Error(err) => {
                self.reset();
                Err(err)
            }
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Protocol decoder status
pub enum DecoderState {
    /// Idle
    Idle,
    /// Receiving data
    Receiving,
    /// Command successfully decoded
    Done,
    /// Error while decoding
    Error(DecodingError),
}
