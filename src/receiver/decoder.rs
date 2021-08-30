use crate::{receiver::DecodingError, Protocol};
use core::fmt::Debug;

/// Protocol decode state machine
pub trait DecoderStateMachine: Protocol {
    /// Decoder state
    type State: DecoderState;
    /// The pulsewidth ranges
    type RangeData: Debug;

    /// Internal State
    type InternalStatus: Into<Status>;

    /// Create the resources
    fn state() -> Self::State;

    /// Create the timer dependent ranges
    /// `resolution`: Timer resolution
    fn ranges(resolution: usize) -> Self::RangeData;

    /// Notify the state machine of a new event
    /// * `edge`: true = positive edge, false = negative edge
    /// * `dt` : Time in micro seconds since last transition
    fn event_full(
        res: &mut Self::State,
        rd: &Self::RangeData,
        edge: bool,
        delta_t: usize,
    ) -> Self::InternalStatus;

    /// Get the command
    /// Returns the data if State == Done, otherwise None
    fn command(state: &Self::State) -> Option<Self::Cmd>;
}

pub trait ConstDecodeStateMachine<const R: usize>: DecoderStateMachine {
    const RANGES: Self::RangeData;

    fn event(res: &mut Self::State, delta_samples: usize, edge: bool) -> Self::InternalStatus {
        Self::event_full(res, &Self::RANGES, edge, delta_samples)
    }
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
