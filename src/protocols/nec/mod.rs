#[macro_use]
pub mod remotes;
pub mod receiver;
pub mod transmitter;

#[cfg(test)]
mod tests;

pub use receiver::{NecError, NecTypeReceiver, NecResult};
pub use transmitter::NecTypeTransmitter;

pub type NecReceiver = NecTypeReceiver<StandardType>;
pub type NecSamsungReceiver = NecTypeReceiver<SamsungType>;

pub type NecTransmitter = NecTypeTransmitter<SamsungType>;

pub struct StandardType;
pub struct SamsungType;


#[derive(Debug, Copy, Clone)]
/// The resulting command
pub struct NecCommand {
    pub addr: u8,
    pub cmd: u8,
}

impl NecCommand {
    pub fn from(bitbuf: u32) -> Self {
        let addr = ((bitbuf) & 0xFF) as u8;
        let cmd = ((bitbuf >> 16) & 0xFF) as u8;
        Self {addr, cmd}
    }
}


pub trait NecTypeTrait {
    const TIMING: Timing;

    fn encode_command(cmd: NecCommand) -> u32;
}

impl NecTypeTrait for StandardType {
    const TIMING: Timing = Timing {
        header_high: 9000,
        header_low: 4500,
        repeat_low: 2250,
        data_high: 560,
        zero_low: 560,
        one_low: 1690,
    };

    fn encode_command(NecCommand {addr, cmd}: NecCommand) -> u32 {
        let addr = u32::from(addr) | u32::from(!addr) << 8;
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }
}

impl NecTypeTrait for SamsungType {
    const TIMING: Timing = Timing {
        header_high: 4500,
        header_low: 4500,
        repeat_low: 2250,
        zero_low: 560,
        data_high: 560,
        one_low: 1690,
    };

    fn encode_command(NecCommand {addr, cmd}: NecCommand) -> u32 {
        // Address is inverted and command is repeated
        let addr = u32::from(addr) | u32::from(addr) << 8;
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }

}

pub struct Timing {
    header_high: u32,
    header_low: u32,
    repeat_low: u32,
    data_high: u32,
    zero_low: u32,
    one_low: u32,
}

