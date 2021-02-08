//! Philips Rc6


mod cmd;
use crate::protocolid::InfraredProtocol;
pub use cmd::Rc6Command;

mod receiver;
mod sender;

#[cfg(test)]
mod tests;

/// Philips Rc6
pub struct Rc6 {}

impl InfraredProtocol for Rc6 {
    type Cmd = Rc6Command;
}
