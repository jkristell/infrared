#[cfg(feature = "remotes")]
use crate::remotecontrol::AsButton;
use crate::ProtocolId;

use core::convert::TryInto;

#[derive(Debug, PartialEq)]
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

    pub fn from_bits(bits: u32, toggle: bool) -> Self {
        let addr = (bits >> 8) as u8;
        let cmd = (bits & 0xFF) as u8;
        Self { addr, cmd, toggle }
    }
}

#[cfg(feature = "remotes")]
impl AsButton for Rc6Command {
    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn protocol(&self) -> ProtocolId {
        ProtocolId::Rc6
    }

    fn create(addr: u32, cmd: u32) -> Option<Self> {
        Some(Rc6Command::new(addr.try_into().ok()?, cmd.try_into().ok()?))
    }
}
