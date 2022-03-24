//! Receiver functionality

mod builder;
//mod constreceiver;
mod bufferinputreceiver;
mod decoder;
mod error;
mod iter;
mod multireceiver;
mod ppoll;
#[allow(clippy::module_inception)]
mod receiver;
pub mod time;

pub use bufferinputreceiver::*;
pub use builder::*;
pub use decoder::*;
pub use error::*;
pub use multireceiver::*;
pub use ppoll::PeriodicPoll;
pub use receiver::*;

/// TODO: Input from `poll` or `event` functions
pub struct NoPinInput;
