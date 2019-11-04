#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ProtocolId {
    Nec = 1,
    Nec16 = 2,
    NecSamsung = 3,
    Rc5 = 4,
    Rc6 = 5,
    /// Samsung 36 bit protocol
    S36 = 6,

    Logging = 31,
}


mod protocols;
pub use protocols::*;

mod transmitter;
pub use transmitter::{TransmitterState, Transmitter};

mod receiver;
pub use receiver::{Receiver, ReceiverState};

#[cfg(feature = "embedded-hal")]
pub mod hal {
    pub use crate::receiver::hal::{Receiver, Receiver2};
    pub use crate::transmitter::PwmTransmitter;
}

#[cfg(feature = "remotes")]
pub mod remotes;

#[cfg(feature = "protocol-dev")]
pub use receiver::ReceiverDebug;

pub mod prelude {
    pub use crate::Receiver;
    pub use crate::Transmitter;
    pub use crate::ReceiverState;
    pub use crate::TransmitterState;
    #[cfg(feature = "embedded-hal")]
    pub use crate::hal;
    #[cfg(feature = "embedded-hal")]
    pub use crate::hal::PwmTransmitter;
}

