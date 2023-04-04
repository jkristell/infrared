use crate::ProtocolId;

/// Command
pub trait Command {
    /// True if command is a repeat
    fn is_repeat(&self) -> bool;
}

/// Command with address and command part
pub trait AddressCommand: Command + Sized {
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
    pub address: u32,
    pub command: u32,
    pub repeat: bool,
}
