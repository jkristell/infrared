//! Infrared protocol

#[cfg(feature = "denon")]
pub mod denon;
#[cfg(feature = "nec")]
pub mod nec;
#[cfg(feature = "rc5")]
pub mod rc5;
#[cfg(feature = "rc6")]
pub mod rc6;
#[cfg(feature = "samsung36")]
pub mod samsung36;

#[cfg(feature = "denon")]
#[doc(inline)]
pub use denon::Denon;
#[cfg(feature = "nec")]
#[doc(inline)]
pub use nec::{AppleNec, Nec, NecExtended, NecRaw, Samsung32};
#[cfg(feature = "rc5")]
#[doc(inline)]
pub use rc5::Rc5;
#[cfg(feature = "rc6")]
#[doc(inline)]
pub use rc6::Rc6;
#[cfg(feature = "samsung36")]
#[doc(inline)]
pub use samsung36::Samsung36;

pub(crate) mod utils;

use core::fmt;

/// Infrared protocol
pub trait Protocol {
    /// The command type decoded and encoded by this protocol
    type Cmd;
}

/// A decoded command of a specific protocol
///
/// All protocol commands expose their content as an address and a command
/// number, the same model used by other infrared libraries.
pub trait Command: Sized {
    /// The protocol of this command
    const PROTOCOL: ProtocolId;

    /// The address (device) part
    fn address(&self) -> u32;

    /// The command (function) part
    fn command(&self) -> u32;

    /// True if the command is a repeat of the previous one
    ///
    /// For RC5 and RC6 this is the toggle bit.
    fn is_repeat(&self) -> bool;

    /// The data bits of the frame, as transmitted (protocol specific)
    fn raw(&self) -> u64;

    /// Create a command from an address and a command number
    ///
    /// Returns `None` if they are out of range for the protocol.
    fn from_parts(address: u32, command: u32) -> Option<Self>;
}

/// A decoded command of any protocol
///
/// Produced by [`MultiReceiver`](crate::receiver::MultiReceiver), and from any
/// protocol command with `Message::from(cmd)`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Message {
    /// The protocol
    pub protocol: ProtocolId,
    /// The address (device) part
    pub address: u32,
    /// The command (function) part
    pub command: u32,
    /// True if the command is a repeat of the previous one
    pub repeat: bool,
    /// The data bits of the frame, as transmitted (protocol specific)
    pub raw: u64,
}

impl<C: Command> From<C> for Message {
    fn from(cmd: C) -> Self {
        Message {
            protocol: C::PROTOCOL,
            address: cmd.address(),
            command: cmd.command(),
            repeat: cmd.is_repeat(),
            raw: cmd.raw(),
        }
    }
}

/// Protocol identifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[non_exhaustive]
pub enum ProtocolId {
    /// NEC, 8 bit address
    Nec = 1,
    /// NEC with 16 bit address
    NecExtended = 2,
    /// Samsung32, NEC variant used by Samsung TVs
    Samsung32 = 3,
    /// Apple NEC variant
    AppleNec = 4,
    /// Philips RC5
    Rc5 = 5,
    /// Philips RC6 (mode 0)
    Rc6 = 6,
    /// Samsung36, used by Samsung Blu-ray players
    Samsung36 = 7,
    /// Denon-K (Kaseikyo family)
    Denon = 8,
}

impl ProtocolId {
    /// Human readable name of the protocol
    pub const fn as_str(self) -> &'static str {
        match self {
            ProtocolId::Nec => "NEC",
            ProtocolId::NecExtended => "NEC extended",
            ProtocolId::Samsung32 => "Samsung32",
            ProtocolId::AppleNec => "Apple NEC",
            ProtocolId::Rc5 => "RC5",
            ProtocolId::Rc6 => "RC6",
            ProtocolId::Samsung36 => "Samsung36",
            ProtocolId::Denon => "Denon-K",
        }
    }
}

impl fmt::Display for ProtocolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned when converting an unknown number into a [`ProtocolId`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UnknownProtocol(pub u8);

impl fmt::Display for UnknownProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown protocol id {}", self.0)
    }
}

impl core::error::Error for UnknownProtocol {}

impl TryFrom<u8> for ProtocolId {
    type Error = UnknownProtocol;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        Ok(match id {
            1 => ProtocolId::Nec,
            2 => ProtocolId::NecExtended,
            3 => ProtocolId::Samsung32,
            4 => ProtocolId::AppleNec,
            5 => ProtocolId::Rc5,
            6 => ProtocolId::Rc6,
            7 => ProtocolId::Samsung36,
            8 => ProtocolId::Denon,
            _ => return Err(UnknownProtocol(id)),
        })
    }
}
