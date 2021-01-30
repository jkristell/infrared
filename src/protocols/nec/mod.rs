//! Nec

mod cmds;
pub mod receiver;
#[cfg(test)]
mod tests;

pub use cmds::{
    NecCommand,
    Nec16Command, NecAppleCommand, NecRawCommand, NecSamsungCommand,
};

#[doc(inline)]
pub use receiver::Nec;
use crate::send::ToPulsedata;

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

    /// Encode the command for sending
    fn pulse_distance(&self, b: &mut [u16]) -> usize {
        b[0] = 0;
        b[1] = Self::PD.hh as u16;
        b[2] = Self::PD.hl as u16;

        let bits = self.pack();

        let mut bi = 3;

        for i in 0..32 {
            let one = (bits >> i) & 1 != 0;
            b[bi] = Self::PD.dh as u16;
            if one {
                b[bi + 1] = Self::PD.ol as u16;
            } else {
                b[bi + 1] = Self::PD.zl as u16;
            }
            bi += 2;
        }

        bi
    }
}

impl<T: NecTiming> ToPulsedata for T {
    fn to_pulsedata(&self, b: &mut [u16]) -> usize {
        self.pulse_distance(b)
    }
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

/*
pub trait NecTiming {
    const PL: &'static NecPulseDistance;
}

impl NecTiming for StandardTiming {
    const PL: &'static NecPulseDistance = &NecPulseDistance {
        hh: 9000,
        hl: 4500,
        rl: 2250,
        dh: 560,
        zl: 560,
        ol: 1690,
    };
}

impl NecTiming for SamsungTiming {
    const PL: &'static NecPulseDistance = &NecPulseDistance {
        hh: 4500,
        hl: 4500,
        rl: 2250,
        zl: 560,
        dh: 560,
        ol: 1690,
    };
}
 */

/// High and low times for Nec-like protocols. In us.
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

