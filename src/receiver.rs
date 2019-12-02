use crate::{Command, ProtocolId};

/// Receiver state machine
pub trait ReceiverStateMachine {
    /// Protocol id
    const ID: ProtocolId;
    /// The resulting command type
    type Cmd: Command;

    /// Create
    fn for_samplerate(samplerate: u32) -> Self;
    /// Add event to state machine
    fn event(&mut self, edge: bool, time: u32) -> ReceiverState<Self::Cmd>;
    /// Reset receiver
    fn reset(&mut self);
}

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum ReceiverState<CMD> {
    Idle,
    Receiving,
    Done(CMD),
    Error(ReceiverError),
    Disabled,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ReceiverError {
    Address(u32),
    Data(u32),
    Other(u32),
}

#[cfg(feature = "protocol-debug")]
pub struct ReceiverDebug<STATE, EXTRA> {
    pub state: STATE,
    pub state_new: STATE,
    pub delta: u16,
    pub extra: EXTRA,
}
