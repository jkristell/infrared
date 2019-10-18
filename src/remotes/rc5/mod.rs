use crate::remotes::RemoteControlCommand;
use crate::rc5::Rc5Command;

mod cdplayer;
pub use cdplayer::Rc5CdPlayer;

impl RemoteControlCommand for Rc5Command {
    fn construct(addr: u16, cmd: u8) -> Self {
        Rc5Command::new(addr as u8, cmd, false)
    }

    fn address(&self) -> u16 {
        self.addr as u16
    }

    fn command(&self) -> u8 {
        self.cmd
    }
}
