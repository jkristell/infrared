/// Command
pub trait Command {
    /// True if command is a repeat
    fn is_repeat(&self) -> bool;
}

/// Command with address and command part
pub trait AddressCommand: Command + Sized {
    /// Get the adress
    fn address(&self) -> u32;
    /// Get the command
    fn command(&self) -> u32;
    /// Create
    fn create(addr: u32, cmd: u32) -> Option<Self>;
}
