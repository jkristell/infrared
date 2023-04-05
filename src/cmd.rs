use crate::ProtocolId;

/// Command
pub trait Command {
    /// True if command is a repeat
    fn is_repeat(&self) -> bool;
}

/// Command with address and command part
pub trait AddressCommand: Command + Sized {
    const ID: ProtocolId;
    fn raw(&self) -> u64 {
        0
    }
    /// Get the address
    fn address(&self) -> u32;
    /// Get the command
    fn command(&self) -> u32;
    /// Create
    fn create(addr: u32, cmd: u32) -> Option<Self>;
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AnyCommand {
    pub protocol: ProtocolId,
    pub raw: u64,
    pub address: u32,
    pub command: u32,
    pub repeat: bool,
}

impl<Cmd: AddressCommand> From<Cmd> for AnyCommand {
    fn from(cmd: Cmd) -> Self {
        AnyCommand {
            protocol: Cmd::ID,
            raw: cmd.raw(),
            address: cmd.address(),
            command: cmd.command(),
            repeat: cmd.is_repeat(),
        }
    }
}
