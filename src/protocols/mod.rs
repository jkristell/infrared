pub mod nec;


pub use nec::receiver::{NecReceiver, NecVariant, NecCommand, NecResult, NecError};

pub use nec::NecType;

pub use nec::transmitter::{NecTransmitter};