#[macro_use]
pub mod receiver;
pub mod transmitter;

#[cfg(test)]
mod tests;

use crate::ProtocolId;
pub use receiver::{NecTypeReceiver};
pub use transmitter::NecTypeTransmitter;

pub struct StandardType;
pub struct SamsungType;
/// NecVariant with 16 bit adress, 8 bit command
pub struct Nec16Type;

pub type NecReceiver = NecTypeReceiver<StandardType>;
pub type NecSamsungReceiver = NecTypeReceiver<SamsungType>;
pub type Nec16Receiver = NecTypeReceiver<Nec16Type>;

pub type NecTransmitter = NecTypeTransmitter<StandardType>;
pub type NecSamsungTransmitter = NecTypeTransmitter<SamsungType>;

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

pub trait NecTypeTrait {
    const PULSEDISTANCE: &'static NecTiming;
    const PROTOCOL: ProtocolId;

    fn encode_command(cmd: NecCommand) -> u32;
    fn decode_command(bits: u32) -> NecCommand;
    fn verify_command(bits: u32) -> bool;
}

impl NecTypeTrait for StandardType {
    const PROTOCOL: ProtocolId = ProtocolId::Nec;
    const PULSEDISTANCE: &'static NecTiming = &STANDARD_DIST;

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

impl NecTypeTrait for Nec16Type {
    const PROTOCOL: ProtocolId = ProtocolId::Nec16;
    const PULSEDISTANCE: &'static NecTiming = &STANDARD_DIST;

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

impl NecTypeTrait for SamsungType {
    const PROTOCOL: ProtocolId = ProtocolId::NecSamsung;
    const PULSEDISTANCE: &'static NecTiming = &NecTiming {
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
