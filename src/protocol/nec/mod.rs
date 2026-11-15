//! NEC and its variants
//!
//! | Type            | Also known as       | Address | Command |
//! |-----------------|---------------------|---------|---------|
//! | [`Nec`]         | NEC1                | 8 bit   | 8 bit   |
//! | [`NecExtended`] | NECext, NEC16       | 16 bit  | 8 bit   |
//! | [`Samsung32`]   | NECx2, Samsung      | 8 bit   | 8 bit   |
//! | [`AppleNec`]    | Apple               | 8 bit   | 12 bit  |
//! | [`NecRaw`]      |                     | -       | -       |

pub mod decoder;
pub mod encoder;

mod apple;
mod extended;
mod raw;
mod samsung;
mod standard;
#[cfg(test)]
mod tests;

use core::marker::PhantomData;

pub use apple::AppleNecCommand;
pub use extended::NecExtendedCommand;
pub use raw::NecRawCommand;
pub use samsung::Samsung32Command;
pub use standard::NecCommand;

use crate::protocol::Protocol;

/// NEC protocol, generic over the command variant
pub struct Nec<C: NecCommandVariant = NecCommand> {
    // Nec Command type
    pub(crate) cmd: PhantomData<C>,
}

impl<C: NecCommandVariant> Protocol for Nec<C> {
    type Cmd = C;
}

/// Samsung32, the NEC variant used by Samsung TVs
pub type Samsung32 = Nec<Samsung32Command>;

/// NEC extended, NEC with 16 bit addresses
pub type NecExtended = Nec<NecExtendedCommand>;

/// NEC variant used by Apple remotes
pub type AppleNec = Nec<AppleNecCommand>;

/// NEC frames without interpretation of the bits, useful for debugging
pub type NecRaw = Nec<NecRawCommand>;

/// Nec Command Variant
pub trait NecCommandVariant: Sized {
    const PULSE_DISTANCE: &'static NecPulseLen;

    /// Repeats are sent as a short repeat frame (header and stop mark).
    /// If false, the whole frame is sent again.
    const SHORT_REPEAT: bool = true;

    /// Validate the bits as a Command of this type
    fn validate(bits: u32) -> bool;

    /// Unpack the bits into Command
    fn unpack(bits: u32, repeat: bool) -> Option<Self>;

    /// Pack command into a u32
    fn pack(&self) -> u32;

    /// True if this is a repeat of the previous command
    fn repeat(&self) -> bool;
}

const NEC_STANDARD_TIMING: &NecPulseLen = &NecPulseLen {
    header_high: 9000,
    header_low: 4500,
    repeat_low: 2250,
    data_high: 560,
    data_zero_low: 560,
    data_one_low: 1690,
};

const NEC_SAMSUNG_TIMING: &NecPulseLen = &NecPulseLen {
    header_high: 4500,
    header_low: 4500,
    repeat_low: 2250,
    data_zero_low: 560,
    data_high: 560,
    data_one_low: 1690,
};

/// High and low times for Nec-like protocol. In us.
#[derive(Copy, Clone)]
pub struct NecPulseLen {
    /// Header high
    header_high: u32,
    /// Header low
    header_low: u32,
    /// Repeat low
    repeat_low: u32,
    /// Data high
    data_high: u32,
    /// Zero low
    data_zero_low: u32,
    /// One low
    data_one_low: u32,
}
