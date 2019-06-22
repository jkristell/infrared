#![no_std]

pub mod nec;
pub mod trace;

/// Receiver trait
pub trait Receiver<T, E> {
    /// Register new event
    fn event(&mut self, timestamp: u32) -> Result<Option<T>, E>;
    /// Reset receiver
    fn reset(&mut self);
    /// Disable receiver
    fn disable(&mut self);
}

