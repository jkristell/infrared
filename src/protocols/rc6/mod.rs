//! Philips Rc6

mod cmd;
pub use cmd::Rc6Command;

mod receiver;
pub use receiver::Rc6;

#[cfg(test)]
mod tests;


