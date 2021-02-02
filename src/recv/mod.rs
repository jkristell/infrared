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
    /// Internal State
    type InternalState: Into<State>;

    /// Create a new Receiver State machine
    fn create() -> Self;

    /// Add event to the state machine
    /// * `edge`: true = positive edge, false = negative edge
    /// * `dt` : Time in micro seconds since last transition
    fn event(&mut self, edge: bool, dt: u32) -> Self::InternalState;

    /// Get the command
    /// Returns the data if State == Done, otherwise None
    fn command(&self) -> Option<Self::Cmd>;

    /// Reset the state machine
    fn reset(&mut self);
}

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum State {
    /// Idle
    Idle,
    /// Receiving data
    Receiving,
    /// Command successfully decoded
    Done,
    /// Error while decoding
    Error(Error),
}

impl Default for State {
    fn default() -> State {
        State::Idle
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
