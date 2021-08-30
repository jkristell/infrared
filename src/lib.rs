//! # Infrared
//!
//! Rust library for using Infrared hardware decoders (For example a Vishay TSOP* decoder),
//! enabling remote control support for embedded project.
//!
//! This library aims for to be useful with the any MCU hal that implements the embedded-hal traits,
//! and at the same time provide functionality for using it with more efficient implementation
//! such as input capture, and be useful in host applications (such as Blipper).
//!
//!
//! ## Examples
//!
//! The [infrared-examples](https://github.com/jkristell/infrared-examples) github repo contains
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

pub use receiver::{ConstReceiver, Receiver};

/// Create a Receiver builder
pub fn builder() -> receiver::Builder {
    receiver::Builder::new()
}

#[doc(inline)]
pub use protocol::{Protocol, ProtocolId};

#[cfg(test)]
#[macro_use]
extern crate std;
