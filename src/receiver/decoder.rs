use crate::receiver::time::InfraMonotonic;
use crate::{receiver::DecodingError, Protocol};
use core::fmt::Debug;

use super::time::PulseSpans;

/// Protocol decode state machine
pub trait DecoderStateMachine<Time: InfraMonotonic>: Protocol {
    /// Decoder state
    type State: DecoderState;
    /// Internal State
    type InternalStatus: Into<Status>;

    const PULSE_LENGTHS: [u32; 8];
    const TOLERANCE: [u32; 8];

    /// Create the resources
    fn state() -> Self::State;

    /// Notify the state machine of a new event
    /// * `edge`: true = positive edge, false = negative edge
    /// * `dt` : Time in micro seconds since last transition
    fn new_event(
        res: &mut Self::State,
        spans: &PulseSpans<Time::Duration>,
        edge: bool,
        dt: Time::Duration,
    ) -> Self::InternalStatus;

    /// Get the command
    /// Returns the data if State == Done, otherwise None
    fn command(state: &Self::State) -> Option<Self::Cmd>;
}

pub trait DecoderState {
    fn reset(&mut self);
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Protocol decoder status
pub enum Status {
    /// Idle
    Idle,
    /// Receiving data
    Receiving,
    /// Command successfully decoded
    Done,
    /// Error while decoding
    Error(DecodingError),
}
