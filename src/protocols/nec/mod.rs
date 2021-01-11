//! Nec

use core::{convert::TryInto, marker::PhantomData};

use crate::{cmd::Protocol, Command};

pub mod receiver;
#[cfg(test)]
mod tests;

#[doc(inline)]
pub use receiver::Nec;

/// Standard Nec protocol
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NecStandard;
/// Nec protocol with Samsung timings
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NecSamsung;
/// Nec with 16 bit address, 8 bit command
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Nec16;

/// Nec Apple
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NecApple;


#[derive(Debug, Copy, Clone, PartialEq)]
/// Nec Command
pub struct NecCommand<VARIANT: NecVariant + ?Sized = NecStandard> {
    pub bitbuf: u32,
    pub addr: u16,
    pub cmd: u8,
    var: PhantomData<VARIANT>,
}

impl<V: NecVariant> NecCommand<V> {
    pub fn new(addr: u16, cmd: u8) -> Self {
        NecCommand {
            bitbuf: 0,
            addr,
            cmd,
            var: PhantomData,
        }
    }
}

impl<VARIANT: NecVariant> Command for NecCommand<VARIANT> {
    fn construct(addr: u32, cmd: u32) -> Option<Self> {
        Some(NecCommand::new(addr.try_into().ok()?, cmd.try_into().ok()?))
    }

    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn data(&self) -> u32 {
        self.cmd.into()
    }

    fn protocol(&self) -> Protocol {
        Protocol::Nec
    }

    fn pulses(&self, b: &mut [u16]) -> usize {
        b[0] = 0;
        b[1] = VARIANT::TIMING.hh as u16;
        b[2] = VARIANT::TIMING.hl as u16;

        let bits = VARIANT::cmd_to_bits(self);

        let mut bi = 3;

        for i in 0..32 {
            let one = (bits >> i) & 1 != 0;
            b[bi] = VARIANT::TIMING.dh as u16;
            if one {
                b[bi + 1] = VARIANT::TIMING.ol as u16;
            } else {
                b[bi + 1] = VARIANT::TIMING.zl as u16;
            }
            bi += 2;
        }

        bi
    }
}

pub trait NecVariant {
    const TIMING: &'static NecTiming;

    fn cmd_to_bits(cmd: &NecCommand<Self>) -> u32;
    fn cmd_from_bits(bits: u32) -> NecCommand<Self>;
    fn cmd_is_valid(bits: u32) -> bool;
}

impl NecVariant for NecStandard {
    const TIMING: &'static NecTiming = &STANDARD_TIMING;

    // Encode to bit
    fn cmd_to_bits(cmd: &NecCommand) -> u32 {
        let addr = cmd.addr;
        let cmd = cmd.cmd;

        let addr = u32::from(addr) | (u32::from(!addr) & 0xFF) << 8;
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }

    fn cmd_from_bits(bits: u32) -> NecCommand<NecStandard> {
        let addr = ((bits) & 0xFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        NecCommand {
            bitbuf: bits,
            addr,
            cmd,
            var: PhantomData,
        }
    }

    fn cmd_is_valid(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0xFF
    }
}

impl NecVariant for Nec16 {
    const TIMING: &'static NecTiming = &STANDARD_TIMING;

    fn cmd_to_bits(cmd: &NecCommand<Self>) -> u32 {
        let addr = u32::from(cmd.addr);
        let cmd = u32::from(cmd.cmd) << 16 | u32::from(!cmd.cmd) << 24;
        addr | cmd
    }

    fn cmd_from_bits(bits: u32) -> NecCommand<Nec16> {
        let addr = ((bits) & 0xFFFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        NecCommand {
            bitbuf: bits,
            addr,
            cmd,
            var: PhantomData,
        }
    }

    fn cmd_is_valid(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF
    }
}

impl NecVariant for NecSamsung {
    const TIMING: &'static NecTiming = &NecTiming {
        hh: 4500,
        hl: 4500,
        rl: 2250,
        zl: 560,
        dh: 560,
        ol: 1690,
    };

    fn cmd_to_bits(cmd: &NecCommand<Self>) -> u32 {
        let addr = u32::from(cmd.addr) | u32::from(cmd.addr) << 8;
        let cmd = u32::from(cmd.cmd) << 16 | u32::from(!cmd.cmd) << 24;
        addr | cmd
    }

    fn cmd_from_bits(bits: u32) -> NecCommand<NecSamsung> {
        let addr = ((bits) & 0xFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        NecCommand {
            bitbuf: bits,
            addr,
            cmd,
            var: PhantomData,
        }
    }

    fn cmd_is_valid(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0
    }
}

impl NecVariant for NecApple {
    const TIMING: &'static NecTiming = &STANDARD_TIMING;

    fn cmd_to_bits(cmd: &NecCommand<Self>) -> u32 {
        0
    }

    fn cmd_from_bits(bits: u32) -> NecCommand<Self> {
        NecCommand {
            bitbuf: bits,
            addr: 0,
            cmd: 0,
            var: PhantomData
        }
    }

    fn cmd_is_valid(bits: u32) -> bool {
        true
    }
}

/// High and low times for Nec-like protocols. In us.
pub struct NecTiming {
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

const STANDARD_TIMING: NecTiming = NecTiming {
    hh: 9000,
    hl: 4500,
    rl: 2250,
    dh: 560,
    zl: 560,
    ol: 1690,
};
