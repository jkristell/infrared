//! # Infrared
//!
//! Rust library for using infrared receiver modules (for example a Vishay TSOP* receiver),
//! enabling remote control support for embedded project.
//!
//! This library aims to be useful with any MCU HAL that implements the embedded-hal traits,
//! and at the same time provide functionality for using it with more efficient implementations
//! such as input capture, and be useful in host applications (such as Blipper).
//!
//!
//! ## Examples
//!
//! The [infrared](https://github.com/jkristell/infrared) github repo contains
//! examples of both Event driven and poll based Receivers, with and without RTIC.
//!

#![no_std]

pub(crate) mod fmt;

pub mod cmd;
pub mod protocol;
pub mod receiver;
pub mod sender;

#[cfg(feature = "remotes")]
pub mod remotecontrol;

#[doc(inline)]
pub use protocol::{Protocol, ProtocolId};
#[cfg(feature = "embedded-hal")]
#[doc(inline)]
pub use receiver::PinReceiver;
#[doc(inline)]
pub use receiver::{BufferReceiver, PeriodicPoll, Receiver};

/// Create a receiver builder
pub fn receiver() -> receiver::Builder {
    receiver::Builder::default()
}

#[cfg(test)]
#[macro_use]
extern crate std;
