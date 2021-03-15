//! Receiver functionality

mod buffer;
pub use buffer::*;
mod event;
pub use event::*;
mod periodic;
use crate::protocol::InfraredProtocol;
pub use periodic::*;

/// Receiver state machine
pub trait InfraredReceiver: InfraredProtocol {
    type ReceiverState: InfraredReceiverState;
    /// Internal State
    type InternalStatus: Into<Status>;

    fn receiver_state(samplerate: u32) -> Self::ReceiverState {
        Self::ReceiverState::create(samplerate)
    }

    /// Add event to the state machine
    /// * `edge`: true = positive edge, false = negative edge
    /// * `dt` : Time in micro seconds since last transition
    fn event(state: &mut Self::ReceiverState, edge: bool, dt: u32) -> Self::InternalStatus;

    /// Get the command
    /// Returns the data if State == Done, otherwise None
    fn command(state: &Self::ReceiverState) -> Option<Self::Cmd>;
}

pub trait InfraredReceiverState {
    fn create(samplerate: u32) -> Self;

    fn reset(&mut self);
}

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum Status {
    /// Idle
    Idle,
    /// Receiving data
    Receiving,
    /// Command successfully decoded
    Done,
    /// Error while decoding
    Error(Error),
}

impl Default for Status {
    fn default() -> Status {
        Status::Idle
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
/// Receive error
pub enum Error {
    /// Error while decoding address
    Address,
    /// Error decoding data bits
    Data,
    /// Validation Error
    Validation,
}
