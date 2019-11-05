use crate::remotes::RemoteControlCommand;
use crate::sbp::SbpCommand;

mod blurayplayer;
pub use blurayplayer::SamsungBluRayPlayer;

impl RemoteControlCommand for SbpCommand {
    fn construct(address: u16, command: u8) -> Self {
        SbpCommand {
            address,
            command,
            valid: true,
        }
    }

    fn address(&self) -> u16 {
        self.address
    }

    fn command(&self) -> u8 {
        self.command
    }
}
