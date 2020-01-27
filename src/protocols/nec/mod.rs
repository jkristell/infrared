//! Nec

pub mod receiver;
pub mod send;

#[cfg(test)]
mod tests;

use crate::Command;
use core::convert::TryInto;
pub use receiver::Nec;
pub use send::NecTypeSender;
use crate::cmd::Protocol;

pub struct NecStandard;
pub struct SamsungVariant;
pub struct Nec16Variant;

/// Nec Samsung variant
pub type NecSamsung = Nec<SamsungVariant>;
/// Nec with 16 bit address, 8 bit command
pub type Nec16 = Nec<Nec16Variant>;

/// Nec - Standard transmitter
pub type NecTransmitter = NecTypeSender<NecStandard>;
/// Nec - Samsung variant transmitter
pub type NecSamsungTransmitter = NecTypeSender<SamsungVariant>;

#[derive(Debug, Copy, Clone, PartialEq)]
/// Nec Command
pub struct NecCommand {
    pub addr: u16,
    pub cmd: u8,
    //pub repeat: bool,
}

impl NecCommand {
    pub fn new(addr: u16, cmd: u8) -> Self {
        NecCommand { addr, cmd }
    }
}

impl Command for NecCommand {
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
}

pub trait NecVariant {
    const TIMING: &'static NecTiming;

    fn encode_command(cmd: NecCommand) -> u32;
    fn decode_command(bits: u32) -> NecCommand;
    fn verify_command(bits: u32) -> bool;
}

impl NecVariant for NecStandard {
    const TIMING: &'static NecTiming = &STANDARD_DIST;

    fn encode_command(NecCommand { addr, cmd }: NecCommand) -> u32 {
        let addr = u32::from(addr) | (u32::from(!addr) & 0xFF) << 8;
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }

    fn decode_command(bits: u32) -> NecCommand {
        let addr = ((bits) & 0xFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        NecCommand { addr, cmd }
    }

    fn verify_command(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0xFF
    }
}

impl NecVariant for Nec16Variant {
    const TIMING: &'static NecTiming = &STANDARD_DIST;

    fn encode_command(NecCommand { addr, cmd }: NecCommand) -> u32 {
        let addr = u32::from(addr);
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }

    fn decode_command(bits: u32) -> NecCommand {
        let addr = ((bits) & 0xFFFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        NecCommand { addr, cmd }
    }

    fn verify_command(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF
    }
}

impl NecVariant for SamsungVariant {
    const TIMING: &'static NecTiming = &NecTiming {
        hh: 4500,
        hl: 4500,
        rl: 2250,
        zl: 560,
        dh: 560,
        ol: 1690,
    };

    fn encode_command(NecCommand { addr, cmd }: NecCommand) -> u32 {
        // Address is inverted and command is repeated
        let addr = u32::from(addr) | u32::from(addr) << 8;
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }

    fn decode_command(bits: u32) -> NecCommand {
        let addr = ((bits) & 0xFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        NecCommand { addr, cmd }
    }

    fn verify_command(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0
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

const STANDARD_DIST: NecTiming = NecTiming {
    hh: 9000,
    hl: 4500,
    rl: 2250,
    dh: 560,
    zl: 560,
    ol: 1690,
};
