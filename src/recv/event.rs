//! Event based Receiver

use crate::recv::{Error, InfraredReceiver, InfraredReceiverState, Status};

/// Event driven receiver
pub struct EventReceiver<Protocol>
where
    Protocol: InfraredReceiver,
{
    pub state: Protocol::ReceiverState,
}

/// Receiver - event based
impl<Protocol: InfraredReceiver> EventReceiver<Protocol> {
    /// Create a new Receiver
    pub fn new(samplerate: u32) -> Self {
        Self {
            state: Protocol::receiver_state(samplerate),
        }
    }

    /// Event happened
    pub fn edge_event<T: Into<u32>>(
        &mut self,
        edge: bool,
        delta_samples: T,
    ) -> Result<Option<Protocol::Cmd>, Error> {
        let delta = delta_samples.into();

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
