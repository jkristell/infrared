//! Receiver functionality

mod buffer;
pub use buffer::*;
mod event;
pub use event::*;
mod periodic;
pub use periodic::*;

/// Receiver state machine
pub trait InfraredReceiver {
    /// The Resulting Command Type
    type Cmd;
    type ReceiverState: InfraredReceiverState;
    /// Internal State
    type InternalState: Into<Status>;

    /// Create a new Receiver State machine
    fn create_receiver() -> Self;

    fn create_receiver_state() -> Self::ReceiverState;

    /// Add event to the state machine
    /// * `edge`: true = positive edge, false = negative edge
    /// * `dt` : Time in micro seconds since last transition
    fn event(&mut self,
             sss: &mut Self::ReceiverState,
             edge: bool, dt: u32) -> Self::InternalState;
}

pub trait InfraredReceiverState {
    /// Get the command
    /// Returns the data if State == Done, otherwise None
    fn command(&self) -> Option<Self::Cmd>;

    /// Reset the state machine
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
    /// Error receiver specific error
    Other,
}
