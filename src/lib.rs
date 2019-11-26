#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ProtocolId {
    /// Nec
    Nec = 1,
    /// Nec with 16 bit address
    Nec16 = 2,
    /// Nec - Samsung variant
    NecSamsung = 3,
    /// Philips Rc5
    Rc5 = 4,
    /// Philips Rc6
    Rc6 = 5,
    /// Samsung 36 bit protocol
    Sbp = 6,

    /// Logging
    Logging = 31,
}

pub trait Command {
    fn construct(addr: u16, cmd: u8) -> Self;
    fn address(&self) -> u16;
    fn command(&self) -> u8;
}


mod protocols;
pub use protocols::*;

pub mod transmitter;
pub mod receiver;


#[cfg(feature = "embedded-hal")]
pub mod hal {
    pub use crate::receiver::hal::{
        HalReceiver,
        HalReceiver2,
        HalReceiver3,
        HalReceiver4,
        HalReceiver5,
    };
    pub use crate::transmitter::PwmTransmitter;
}

#[cfg(feature = "remotes")]
pub mod remotes;

#[cfg(feature = "protocol-dev")]
pub use receiver::ReceiverDebug;

pub mod prelude {
    pub use crate::{Command, ProtocolId};
    pub use crate::transmitter::{TransmitterState, Transmitter};
    pub use crate::receiver::{ReceiverStateMachine, ReceiverState};
    #[cfg(feature = "embedded-hal")]
    pub use crate::hal;
}

