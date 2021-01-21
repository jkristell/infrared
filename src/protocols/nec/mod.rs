//! Nec

pub mod cmds;
pub mod receiver;
#[cfg(test)]
mod tests;

use crate::protocols::nec::cmds::{
    Nec16Command, NecAppleCommand, NecRawCommand, NecSamsungCommand,
};

#[doc(inline)]
pub use receiver::Nec;

/// Standard Nec protocol timing
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StandardTiming;

/// Nec protocol with Samsung timings
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SamsungTiming;

/// Nec variant with Samsung bit encoding and Samsung timing
pub type NecSamsung = Nec<NecSamsungCommand, SamsungTiming>;

/// Nec variant with 16 bit address and Nec standard timing
pub type Nec16 = Nec<Nec16Command>;

/// Nec variant with Apple specific bit encoding and Standard timing
pub type NecApple = Nec<NecAppleCommand>;

/// Nec variant without any specific bit unpacking, useful for debugging
pub type NecDebug = Nec<NecRawCommand>;

/// Nec Command bit fiddling Trait
pub trait NecCommandTrait<Timing: NecTiming>: Sized {
    /// Validate the bits as a Command of this type
    fn validate(bits: u32) -> bool;

    /// Unpack the bits into Command
    fn unpack(bits: u32, repeat: bool) -> Option<Self>;

    /// Pack command into a u32
    fn pack(&self) -> u32;

    /// Pulselengths for Command
    fn to_pulselengths(&self, b: &mut [u16]) -> usize {
        b[0] = 0;
        b[1] = Timing::PL.hh as u16;
        b[2] = Timing::PL.hl as u16;

        let bits = self.pack();

        let mut bi = 3;

        for i in 0..32 {
            let one = (bits >> i) & 1 != 0;
            b[bi] = Timing::PL.dh as u16;
            if one {
                b[bi + 1] = Timing::PL.ol as u16;
            } else {
                b[bi + 1] = Timing::PL.zl as u16;
            }
            bi += 2;
        }

        bi
    }
}

pub trait NecTiming {
    const PL: &'static NecPulselengths;
}

impl NecTiming for StandardTiming {
    const PL: &'static NecPulselengths = &NecPulselengths {
        hh: 9000,
        hl: 4500,
        rl: 2250,
        dh: 560,
        zl: 560,
        ol: 1690,
    };
}

impl NecTiming for SamsungTiming {
    const PL: &'static NecPulselengths = &NecPulselengths {
        hh: 4500,
        hl: 4500,
        rl: 2250,
        zl: 560,
        dh: 560,
        ol: 1690,
    };
}

/// High and low times for Nec-like protocols. In us.
pub struct NecPulselengths {
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
