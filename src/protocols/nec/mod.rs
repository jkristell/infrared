#[macro_use]
pub mod receiver;
pub mod transmitter;

#[cfg(test)]
mod tests;

use crate::{ProtocolId, Command};
pub use receiver::{NecType};
pub use transmitter::NecTypeTransmitter;

pub struct NecStandard;
pub struct SamsungVariant;
/// NecVariant with 16 bit adress, 8 bit command
pub struct Nec16Variant;

pub type Nec = NecType<NecStandard>;
pub type NecSamsung = NecType<SamsungVariant>;
pub type Nec16 = NecType<Nec16Variant>;

pub type NecTransmitter = NecTypeTransmitter<NecStandard>;
pub type NecSamsungTransmitter = NecTypeTransmitter<SamsungVariant>;

#[derive(Debug, Copy, Clone, PartialEq)]
/// A Nec Command
pub struct NecCommand {
    pub addr: u16,
    pub cmd: u8,
}

impl NecCommand {
    pub fn new(addr: u16, cmd: u8) -> Self {
        NecCommand { addr, cmd }
    }
}

impl Command for NecCommand {
    fn construct(addr: u16, cmd: u8) -> Self {
        NecCommand::new(addr, cmd)
    }

    fn address(&self) -> u16 {
        self.addr as u16
    }

    fn command(&self) -> u8 {
        self.cmd
    }
}

pub trait NecVariant {
    const TIMING: &'static NecTiming;
    const PROTOCOL: ProtocolId;

    fn encode_command(cmd: NecCommand) -> u32;
    fn decode_command(bits: u32) -> NecCommand;
    fn verify_command(bits: u32) -> bool;
}

impl NecVariant for NecStandard {
    const PROTOCOL: ProtocolId = ProtocolId::Nec;
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
    const PROTOCOL: ProtocolId = ProtocolId::Nec16;
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
    const PROTOCOL: ProtocolId = ProtocolId::NecSamsung;
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
