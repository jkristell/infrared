use core::fmt::Debug;

use crate::{
    receiver::{time::InfraMonotonic, DecodingError, time::PulseSpans},
    Protocol,
};

pub trait DecoderAdapter<Mono: InfraMonotonic>: Protocol {
    /// Type of the decoder
    type Decoder: ProtocolDecoder<Mono, Self::Cmd>;

    /// Pulse lenghts
    const PULSE: [u32; 8];
    /// Pulse length tolerances
    const TOL: [u32; 8];

    /// Create the decoder
    fn decoder(freq: u32) -> Self::Decoder;

    fn create_pulsespans(freq: u32) -> PulseSpans<Mono> {
        PulseSpans::new(freq, &Self::PULSE, &Self::TOL)
    }
}

/// Protocol decode state machine
pub trait ProtocolDecoder<Mono: InfraMonotonic, Cmd> {
    /// Notify the state machine of a new event
    /// * `edge`: true = positive edge, false = negative edge
    /// * `dt` : Duration since last event
    fn event(&mut self, edge: bool, dt: Mono::Duration) -> State;

    /// Get the command
    /// Returns the data if State == Done, otherwise None
    fn command(&self) -> Option<Cmd>;

    /// Reset the decoder
    fn reset(&mut self);

    /// Get the time spans
    fn spans(&self) -> &PulseSpans<Mono>;

    fn combined(&mut self, edge: bool, dt: Mono::Duration) -> Result<Option<Cmd>, DecodingError> {
        match self.event(edge, dt) {
            State::Idle | State::Receiving => Ok(None),
            State::Done => {
                let cmd = self.command();
                self.reset();
                Ok(cmd)
            }
            State::Error(err) => {
                self.reset();
                Err(err)
            }
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Protocol decoder status
pub enum State {
    /// Idle
    Idle,
    /// Receiving data
    Receiving,
    /// Command successfully decoded
    Done,
    /// Error while decoding
    Error(DecodingError),
}
