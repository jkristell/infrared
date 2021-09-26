//! Nec

pub mod decoder;
pub mod encoder;

mod apple;
mod nec16;
mod raw;
mod samsung;
mod standard;
#[cfg(test)]
mod tests;

use crate::protocol::Protocol;
use core::marker::PhantomData;

pub use apple::NecAppleCommand;
pub use nec16::Nec16Command;
pub use raw::NecDebugCmd;
pub use samsung::NecSamsungCommand;
pub use standard::NecCommand;

/// Nec Receiver with Nec standard bit encoding and Standard timing
pub struct Nec<C: NecCommandVariant = NecCommand> {
    // Nec Command type
    pub(crate) cmd: PhantomData<C>,
}

impl<C: NecCommandVariant> Protocol for Nec<C> {
    type Cmd = C;
}

/// Nec variant with Samsung bit encoding and Samsung timing
pub type NecSamsung = Nec<NecSamsungCommand>;

/// Nec variant with 16 bit address and Nec standard timing
pub type Nec16 = Nec<Nec16Command>;

/// Nec variant with Apple specific bit encoding and Standard timing
pub type NecApple = Nec<NecAppleCommand>;

/// Nec variant without any specific bit unpacking, useful for debugging
pub type NecDebug = Nec<NecDebugCmd>;

/// Nec Command Variant
pub trait NecCommandVariant: Sized {
    const PULSE_DISTANCE: &'static NecPulseDistance;

    /// Validate the bits as a Command of this type
    fn validate(bits: u32) -> bool;

    /// Unpack the bits into Command
    fn unpack(bits: u32, repeat: bool) -> Option<Self>;

    /// Pack command into a u32
    fn pack(&self) -> u32;
}

pub(crate) const NEC_STANDARD_TIMING: &NecPulseDistance = &NecPulseDistance {
    header_high: 9000,
    header_low: 4500,
    repeat_low: 2250,
    data_high: 560,
    data_zero_low: 560,
    data_one_low: 1690,
};

pub(crate) const NEC_SAMSUNG_TIMING: &NecPulseDistance = &NecPulseDistance {
    header_high: 4500,
    header_low: 4500,
    repeat_low: 2250,
    data_zero_low: 560,
    data_high: 560,
    data_one_low: 1690,
};

/// High and low times for Nec-like protocol. In us.
#[derive(Copy, Clone)]
pub struct NecPulseDistance {
    /// Header high
    header_high: usize,
    /// Header low
    header_low: usize,
    /// Repeat low
    repeat_low: usize,
    /// Data high
    data_high: usize,
    /// Zero low
    data_zero_low: usize,
    /// One low
    data_one_low: usize,
}
