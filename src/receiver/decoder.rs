use crate::receiver::time::InfraMonotonic;
use crate::{receiver::DecodingError, Protocol};
use core::fmt::Debug;

use super::time::PulseSpans;

/// Protocol decode state machine
pub trait DecoderStateMachine<Mono: InfraMonotonic>: Protocol {
    /// Decoder state
    type Data: DecoderData;
    /// Internal State type
    type InternalState: Into<State>;

    const PULSE: [u32; 8];
    const TOL: [u32; 8];

    fn create_pulsespans(freq: u32) -> PulseSpans<Mono::Duration> {
        PulseSpans {
            spans: [
                Mono::create_span(freq, Self::PULSE[0], Self::TOL[0]),
                Mono::create_span(freq, Self::PULSE[1], Self::TOL[1]),
                Mono::create_span(freq, Self::PULSE[2], Self::TOL[2]),
                Mono::create_span(freq, Self::PULSE[3], Self::TOL[3]),
                Mono::create_span(freq, Self::PULSE[4], Self::TOL[4]),
                Mono::create_span(freq, Self::PULSE[5], Self::TOL[5]),
                Mono::create_span(freq, Self::PULSE[6], Self::TOL[6]),
                Mono::create_span(freq, Self::PULSE[7], Self::TOL[7]),
            ],
        }
    }

    /// Create the resources
    fn create_data() -> Self::Data;

    /// Notify the state machine of a new event
    /// * `edge`: true = positive edge, false = negative edge
    /// * `dt` : Time in micro seconds since last transition
    fn event(
        data: &mut Self::Data,
        spans: &PulseSpans<Mono::Duration>,
        edge: bool,
        dt: Mono::Duration,
    ) -> Self::InternalState;

    /// Get the command
    /// Returns the data if State == Done, otherwise None
    fn command(data: &Self::Data) -> Option<Self::Cmd>;
}

pub trait DecoderData {
    fn reset(&mut self);
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
