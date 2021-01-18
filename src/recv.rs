//! Receiver

/// Event driven receiver
pub struct EventReceiver<PROTOCOL> {
    pub sm: PROTOCOL,
    /// Receiver running at samplerate
    precalc_multiplier: u32,
}

/// Receiver - event based
impl<PROTOCOL: ReceiverSM> EventReceiver<PROTOCOL> {
    /// Create a new Receiver
    pub fn new(samplerate: u32) -> Self {
        Self {
            sm: PROTOCOL::create(),
            precalc_multiplier: crate::TIMEBASE / samplerate,
        }
    }

    /// Event happened
    pub fn edge_event<T: Into<u32>>(
        &mut self,
        edge: bool,
        delta_samples: T,
    ) -> Result<Option<PROTOCOL::Cmd>, Error> {
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

/// Receiver to use with periodic polling
pub struct PeriodicReceiver<PROTOCOL> {
    pub recv: EventReceiver<PROTOCOL>,
    /// Last seen edge
    edge: bool,
    /// Seen at
    last: u32,
}

impl<PROTOCOL: ReceiverSM> PeriodicReceiver<PROTOCOL> {
    pub fn new(samplerate: u32) -> Self {
        Self {
            recv: EventReceiver::new(samplerate),
            edge: false,
            last: 0,
        }
    }

    pub fn poll(&mut self, edge: bool, ts: u32) -> Result<Option<PROTOCOL::Cmd>, Error> {
        if self.edge == edge {
            return Ok(None);
        }

        let dt = ts.wrapping_sub(self.last);

        self.last = ts;
        self.edge = edge;
        self.recv.edge_event(edge, dt)
    }

    pub fn reset(&mut self) {
        self.recv.reset()
    }
}

/// Receiver state machine
pub trait ReceiverSM {
    /// The Resulting Command Type
    type Cmd;
    /// Internal State
    type InternalState: Into<State>;

    /// Create a new ReceiverSM
    fn create() -> Self;

    /// Add event to the state machine
    /// * `edge`: true = positive edge, false = negative edge
    /// * `dt` : Time in micro seconds since last transition
    fn event(&mut self, edge: bool, dt: u32) -> Self::InternalState;

    /// Get the command
    /// Returns the data if State == Done, otherwise None
    fn command(&self) -> Option<Self::Cmd>;

    /// Reset the state machine
    fn reset(&mut self);
}

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum State {
    /// Idle
    Idle,
    /// Receiving data
    Receiving,
    /// Command successfully decoded
    Done,
    /// Error while decoding
    Error(Error),
}

impl Default for State {
    fn default() -> State {
        State::Idle
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
/// Receive error
pub enum Error {
    /// Error while decoding address
    Address,
    /// Error decoding data bits
    Data,
    /// Error receiver specific error
    Other,
}
