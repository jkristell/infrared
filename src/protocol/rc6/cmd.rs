use crate::cmd::{AddressCommand, Command};

use core::convert::TryInto;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Rc6Command {
    pub addr: u8,
    pub cmd: u8,
    pub toggle: bool,
}

impl Rc6Command {
    pub fn new(addr: u8, cmd: u8) -> Self {
        Self {
            addr,
            cmd,
            toggle: false,
        }
    }

    pub fn from_bits(bits: u16, toggle: bool) -> Self {
        let addr = (bits >> 8) as u8;
        let cmd = (bits & 0xFF) as u8;
        Self { addr, cmd, toggle }
    }
}

impl Command for Rc6Command {
    fn is_repeat(&self) -> bool {
        self.toggle
    }
}

impl AddressCommand for Rc6Command {
    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn create(addr: u32, cmd: u32) -> Option<Self> {
        Some(Rc6Command::new(addr.try_into().ok()?, cmd.try_into().ok()?))
    }
}
