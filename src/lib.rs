#![no_std]

/// NEC protocol decoder
pub mod nec;
/// Tracing protocol decoder
pub mod trace;

/// Remote controls
pub mod remotes;

#[derive(PartialEq)]
/// Protocol decoder state
pub enum State<T, E> {
    Idle,
    Receiving,
    Done(T),
    Err(E),
}

/// Receiver trait
pub trait Receiver<T, E> {
    /// Register new event
    fn event(&mut self, rising: bool, timestamp: u32) -> State<T, E>;
    /// Reset receiver
    fn reset(&mut self);
    /// Disable receiver
    fn disable(&mut self);
}

impl<T, E> State<T, E> {
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
