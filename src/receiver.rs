//! Receiver functionality

mod builder;
//mod constreceiver;
mod decoder;
mod error;
mod iter;
mod multireceiver;
mod bufferinputreceiver;
mod ppoll;
#[allow(clippy::module_inception)]
mod receiver;
pub mod time;

pub use builder::*;
pub use decoder::*;
pub use error::*;
pub use multireceiver::*;
pub use receiver::*;
pub use bufferinputreceiver::*;
pub use ppoll::PeriodicPoll;

/// Input from `poll` or `event` functions
pub struct DefaultInput;
/// Input from pin
pub struct PinInput<P>(pub P);
