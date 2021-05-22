//! Event based Receiver

use crate::recv::{Error, InfraredReceiver, InfraredReceiverState, Status};

/// Event driven receiver
pub struct EventReceiver<Protocol>
where
    Protocol: InfraredReceiver,
{
    pub state: Protocol::ReceiverState,
}

impl<Protocol: InfraredReceiver> EventReceiver<Protocol> {
    /// Create a new Receiver
    pub fn new(resolution: u32) -> Self {
        Self {
            state: Protocol::receiver_state(resolution),
        }
    }

    /// Event happened
    pub fn update(
        &mut self,
        edge: bool,
        delta: u32,
    ) -> Result<Option<Protocol::Cmd>, Error> {

        // Update state machine
        let state: Status = Protocol::event(&mut self.state, edge, delta).into();

        match state {
            Status::Done => {
                let cmd = Protocol::command(&mut self.state);
                self.state.reset();
                Ok(cmd)
            }
            Status::Error(err) => {
                self.state.reset();
                Err(err)
            }
            Status::Idle | Status::Receiving => Ok(None),
        }
    }

    /// Reset receiver
    pub fn reset(&mut self) {
        self.state.reset();
    }
}
