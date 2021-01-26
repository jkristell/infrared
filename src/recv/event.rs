//! Event based Receiver

use crate::recv::{Error, ReceiverSM, State};

/// Event driven receiver
pub struct EventReceiver<Protocol> {
    pub sm: Protocol,
    /// Receiver running at samplerate
    precalc_multiplier: u32,
}

/// Receiver - event based
impl<Protocol: ReceiverSM> EventReceiver<Protocol> {
    /// Create a new Receiver
    pub fn new(samplerate: u32) -> Self {
        Self {
            sm: Protocol::create(),
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
        let state: State = self.sm.event(edge, dt_us).into();

        match state {
            State::Done => {
                let cmd = self.sm.command();
                self.sm.reset();
                Ok(cmd)
            }
            State::Error(err) => {
                self.sm.reset();
                Err(err)
            }
            State::Idle | State::Receiving => Ok(None),
        }
    }

    /// Reset receiver
    pub fn reset(&mut self) {
        self.sm.reset();
    }
}

