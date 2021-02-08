//! Event based Receiver

use crate::recv::{Error, InfraredReceiver, Status, InfraredReceiverState};

/// Event driven receiver
pub struct EventReceiver<Protocol>
where
    Protocol: InfraredReceiver,
{
    pub receiver_state: Protocol::ReceiverState,
    /// Receiver running at samplerate
    precalc_multiplier: u32,
}

/// Receiver - event based
impl<Protocol: InfraredReceiver> EventReceiver<Protocol> {
    /// Create a new Receiver
    pub fn new(samplerate: u32) -> Self {
        Self {
            receiver_state: Protocol::receiver_state(samplerate),
            precalc_multiplier: crate::TIMEBASE / samplerate,
        }
    }

    /// Event happened
    pub fn edge_event<T: Into<u32>>(
        &mut self,
        edge: bool,
        delta_samples: T,
    ) -> Result<Option<Protocol::Cmd>, Error> {
        // Convert to micro seconds
        let dt_us = delta_samples.into() * self.precalc_multiplier;

        // Update state machine
        let state: Status = Protocol::event(
            &mut self.receiver_state,
            edge, dt_us).into();

        match state {
            Status::Done => {
                let cmd = Protocol::command(&mut self.receiver_state);
                self.receiver_state.reset();
                Ok(cmd)
            }
            Status::Error(err) => {
                self.receiver_state.reset();
                Err(err)
            }
            Status::Idle | Status::Receiving => Ok(None),
        }
    }

    /// Reset receiver
    pub fn reset(&mut self) {
        self.receiver_state.reset();
    }
}

