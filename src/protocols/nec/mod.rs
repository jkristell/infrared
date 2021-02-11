//! Nec

mod cmds;
pub mod receiver;
pub mod sender;
#[cfg(test)]
mod tests;

pub use cmds::{Nec16Command, NecAppleCommand, NecCommand, NecRawCommand, NecSamsungCommand};

use crate::protocolid::InfraredProtocol;
use core::marker::PhantomData;

/// Nec Receiver with Nec standard bit encoding and Standard timing
pub struct Nec<C: NecCommandTrait = NecCommand> {
    // Nec Command type
    pub(crate) cmd: PhantomData<C>,
}

impl<C: NecCommandTrait> InfraredProtocol for Nec<C> {
    type Cmd = C;
}

/// Nec variant with Samsung bit encoding and Samsung timing
pub type NecSamsung = Nec<NecSamsungCommand>;

/// Nec variant with 16 bit address and Nec standard timing
pub type Nec16 = Nec<Nec16Command>;

/// Nec variant with Apple specific bit encoding and Standard timing
pub type NecApple = Nec<NecAppleCommand>;

/// Nec variant without any specific bit unpacking, useful for debugging
pub type NecDebug = Nec<NecRawCommand>;

/// Nec Command bit fiddling Trait
pub trait NecCommandTrait: Sized {
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

/// High and low times for Nec-like protocols. In us.
#[derive(Copy, Clone)]
pub struct NecPulseDistance {
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
