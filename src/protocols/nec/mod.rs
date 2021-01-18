//! Nec

pub mod cmds;
pub mod receiver;
#[cfg(test)]
mod tests;

#[doc(inline)]
pub use receiver::Nec;
use crate::protocols::nec::cmds::{Nec16Command, NecSamsungCommand, NecAppleCommand};

/// Standard Nec protocol
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StandardTiming;

/// Nec protocol with Samsung timings
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SamsungTiming;

pub type NecSamsung = Nec<NecSamsungCommand, SamsungTiming>;

pub type Nec16 = Nec<Nec16Command>;

pub type NecDebug = Nec<NecRaw>;

pub type NecApple = Nec<NecAppleCommand>;


#[derive(Debug, Copy, Clone, PartialEq)]
/// Nec Command
pub struct NecRaw {
    pub bits: u32,
}

impl<T: NecTiming> NecCommandTrait<T> for NecRaw {
    fn validate<B: Into<u32>>(_bits: B) -> bool {
        true
    }

    fn unpack(cmd: NecRaw, _repeat: bool) -> Option<Self> {
        Some(cmd)
    }

    fn pack(&self) -> NecRaw {
        self.clone()
    }
}

impl From<u32> for NecRaw {
    fn from(bits: u32) -> Self {
        NecRaw { bits }
    }
}

impl From<NecRaw> for u32 {
    fn from(nr: NecRaw) -> Self {
        nr.bits
    }
}

/// Nec Command Trait
///
pub trait NecCommandTrait<Timing: NecTiming>: Sized {
    fn validate<T: Into<u32>>(bits: T) -> bool;

    fn validate_self(&self) -> bool {
        Self::validate(self.pack())
    }

    fn unpack(cmd: NecRaw, repeat: bool) -> Option<Self>;

    fn pack(&self) -> NecRaw;

    fn to_pulselengths(&self, b: &mut[u16]) -> usize {
        b[0] = 0;
        b[1] = Timing::PL.hh as u16;
        b[2] = Timing::PL.hl as u16;

        let bits = self.pack().bits;

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

