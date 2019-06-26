#![no_std]

/// NEC protocol decoder
pub mod nec;
/// Tracing protocol decoder
pub mod trace;

/// Remote controls
pub mod remotes;
pub use remotes::Remote;

#[derive(PartialEq)]
/// Protocol decoder state
pub enum State<CMD, ERR> {
    Idle,
    Receiving,
    Done(CMD),
    Err(ERR),
}

/// Receiver trait
pub trait Receiver {
    /// The resulting command type
    type Command;
    /// Receive Error
    type ReceiveError;

    /// Register new event
    fn event(&mut self, rising: bool, timestamp: u32) -> State<Self::Command, Self::ReceiveError>;
    /// Reset receiver
    fn reset(&mut self);
    /// Disable receiver
    fn disable(&mut self);
}

impl<CMD, ERR> State<CMD, ERR> {
    pub fn is_err(&self) -> bool {
        match *self {
            State::Err(_) => true,
            _ => false,
        }
    }
    pub fn is_done(&self) -> bool {
        match *self {
            State::Done(_) => true,
            _ => false,
        }
    }
}
