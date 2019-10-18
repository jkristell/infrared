use crate::remotes::RemoteControlCommand;
use crate::rc6::Rc6Command;

impl RemoteControlCommand for Rc6Command {
    fn construct(addr: u16, cmd: u8) -> Self {
        Rc6Command::new(addr as u8, cmd)
    }

    fn address(&self) -> u16 {
        self.addr as u16
    }

    fn command(&self) -> u8 {
        self.cmd
    }
}
