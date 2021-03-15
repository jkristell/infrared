//! Receiver functionality

mod builder;
mod constreceiver;
mod decoder;
mod error;
mod iter;
mod multireceiver;
mod receiver;

pub use builder::*;
pub use constreceiver::*;
pub use decoder::*;
pub use error::*;
pub use multireceiver::*;
pub use receiver::*;

/// Input from `poll` or `event` functions
pub struct DefaultInput;
/// Input from pin
pub struct PinInput<P>(pub P);
/// Input from buffer
pub struct BufferInput<'a>(&'a [usize]);

#[derive(Default)]
/// Periodic Poll
pub struct Poll {
    clock: usize,
    /// Last seen edge
    edge: bool,
    /// Seen at
    last_edge: usize,
}

#[derive(Default)]
/// Event driven
pub struct Event {}
