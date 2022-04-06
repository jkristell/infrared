//! Infrared protocol

mod capture;

#[cfg(feature = "denon")]
pub mod denon;
#[cfg(feature = "nec")]
pub mod nec;
#[cfg(feature = "rc5")]
pub mod rc5;
#[cfg(feature = "rc6")]
pub mod rc6;
#[cfg(feature = "sbp")]
pub mod sbp;

pub use capture::Capture;
#[cfg(feature = "denon")]
#[doc(inline)]
pub use denon::{Denon};
#[cfg(feature = "nec")]
#[doc(inline)]
pub use nec::{
    AppleNec, Nec, Nec16, NecDebug, SamsungNec,
};
#[cfg(feature = "rc5")]
#[doc(inline)]
pub use rc5::{Rc5};
#[cfg(feature = "rc6")]
#[doc(inline)]
pub use rc6::{Rc6, };
#[cfg(feature = "sbp")]
#[doc(inline)]
pub use sbp::{Sbp};

pub(crate) mod utils;

/// Infrared protocol
pub trait Protocol {
    type Cmd;
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
