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
pub struct Nec<C = NecCommand> {
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
    /// Validate the bits as a Command of this type
    fn validate(bits: u32) -> bool;

    /// Unpack the bits into Command
    fn unpack(bits: u32, repeat: bool) -> Option<Self>;

    /// Pack command into a u32
    fn pack(&self) -> u32;
}

pub trait NecTiming: NecCommandTrait {
    const PD: &'static NecPulseDistance;
}

pub(crate) const NEC_STANDARD_TIMING: &'static NecPulseDistance = &NecPulseDistance {
    hh: 9000,
    hl: 4500,
    rl: 2250,
    dh: 560,
    zl: 560,
    ol: 1690,
};

pub(crate) const NEC_SAMSUNG_TIMING: &'static NecPulseDistance = &NecPulseDistance {
    hh: 4500,
    hl: 4500,
    rl: 2250,
    zl: 560,
    dh: 560,
    ol: 1690,
};

/// High and low times for Nec-like protocols. In us.
#[derive(Copy, Clone)]
pub struct NecPulseDistance {
    /// Header high
    hh: u32,
    /// Header low
    hl: u32,
    /// Repeat low
    rl: u32,
    /// Data high
    dh: u32,
    /// Zero low
    zl: u32,
    /// One low
    ol: u32,
}
