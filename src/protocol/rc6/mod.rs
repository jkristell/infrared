//! Philips Rc6

use crate::protocol::Protocol;

mod cmd;
mod decoder;
mod encoder;
#[cfg(test)]
mod tests;

pub use cmd::Rc6Command;

/// Philips Rc6
pub struct Rc6 {}

impl Protocol for Rc6 {
    type Cmd = Rc6Command;
}
