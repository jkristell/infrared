use crate::remotes::RemoteControlCommand;
use crate::nec::NecCommand;

mod samsungtv;
mod specialformp3;
pub use samsungtv::SamsungTv;
pub use specialformp3::SpecialForMp3;

impl RemoteControlCommand for NecCommand {
    fn construct(addr: u16, cmd: u8) -> Self {
        NecCommand::new(addr as u8, cmd)
    }

    fn address(&self) -> u16 {
        self.addr as u16
    }

    fn command(&self) -> u8 {
        self.cmd
    }
}
