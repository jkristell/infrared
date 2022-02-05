//! Infrared protocol

#[cfg(feature = "denon")]
mod denon;
#[cfg(feature = "nec")]
mod nec;
#[cfg(feature = "rc5")]
mod rc5;
#[cfg(feature = "rc6")]
mod rc6;
#[cfg(feature = "sbp")]
mod sbp;

mod mitshubishi;

#[cfg(feature = "denon")]
#[doc(inline)]
pub use denon::{Denon, DenonCommand};
#[cfg(feature = "nec")]
#[doc(inline)]
pub use nec::{
    Nec, Nec16, Nec16Command, NecApple, NecAppleCommand, NecCommand, NecDebug, NecDebugCmd,
    NecSamsung, NecSamsungCommand,
};
#[cfg(feature = "rc5")]
#[doc(inline)]
pub use rc5::{Rc5, Rc5Command};
#[cfg(feature = "rc6")]
#[doc(inline)]
pub use rc6::{Rc6, Rc6Command};
#[cfg(feature = "sbp")]
#[doc(inline)]
pub use sbp::{Sbp, SbpCommand};

pub use mitshubishi::{Mitsubishi, MCmd};

pub mod capture;

pub(crate) mod utils;

/// Infrared protocol
pub trait Protocol {
    type Cmd;
}

pub struct DummyProtocol {}

impl Protocol for DummyProtocol {
    type Cmd = ();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[non_exhaustive]
/// Protocol id
pub enum ProtocolId {
    /// Standard Nec
    Nec = 1,
    /// Nec with 16 bit addresses
    Nec16 = 2,
    /// Nec (Samsung variant)
    NecSamsung = 3,
    /// Nec (Apple variant)
    NecApple = 4,
    /// Philips Rc5
    Rc5 = 5,
    /// Philips Rc6
    Rc6 = 6,
    /// Samsung Blu-ray player protocol
    Sbp = 7,
    /// Denon
    Denon = 8,
    /// Placeholder
    Unknown = 255,
}

impl From<u8> for ProtocolId {
    fn from(u: u8) -> Self {
        match u {
            1 => ProtocolId::Nec,
            2 => ProtocolId::Nec16,
            3 => ProtocolId::NecSamsung,
            4 => ProtocolId::NecApple,
            5 => ProtocolId::Rc5,
            6 => ProtocolId::Rc6,
            7 => ProtocolId::Sbp,
            8 => ProtocolId::Denon,
            _ => ProtocolId::Unknown,
        }
    }
}
