//! Receiver functionality

mod builder;
//mod constreceiver;
mod decoder;
mod error;
mod iter;
mod multireceiver;
mod bufferinputreceiver;
#[allow(clippy::module_inception)]
mod receiver;
pub mod time;

pub use builder::*;
pub use decoder::*;
pub use error::*;
pub use multireceiver::*;
pub use receiver::*;
pub use bufferinputreceiver::*;

/// Input from `poll` or `event` functions
pub struct DefaultInput;
/// Input from pin
pub struct PinInput<P>(pub P);

#[derive(Default)]
/// Periodic Poll
pub struct Poll {
    clock: u32,
    /// Last seen edge
    edge: bool,
    /// Seen at
    last_edge: u32,
}

#[derive(Default)]
/// Event driven
pub struct Event {}
