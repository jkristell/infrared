//! Receiver

use crate::Command;

/// Event driven receiver
pub struct EventReceiver<SM> {
    pub sm: SM,
    /// Receiver running at samplerate
    precalc_multiplier: u32,
}

/// Receiver - event based
impl<SM: ReceiverSM> EventReceiver<SM> {
    /// Create a new Receiver
    pub fn new(samplerate: u32) -> Self {
        Self {
            sm: SM::create(),
            precalc_multiplier: 1_000_000 / samplerate,
        }
    }

    /// Event happened
    pub fn edge_event(&mut self, edge: bool, delta_samples: u32) -> Result<Option<SM::Cmd>, Error> {
        // Convert to micro seconds
        let dt_us = delta_samples * self.precalc_multiplier;

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
pub struct PeriodicReceiver<SM> {
    pub recv: EventReceiver<SM>,
    /// Last seen edge
    edge: bool,
    /// Seen at
    last: u32,
}

impl<SM: ReceiverSM> PeriodicReceiver<SM> {
    pub fn new(samplerate: u32) -> Self {
        Self {
            recv: EventReceiver::new(samplerate),
            edge: false,
            last: 0,
        }
    }

    pub fn poll(&mut self, edge: bool, ts: u32) -> Result<Option<SM::Cmd>, Error> {
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

/// Receiver for decoding a captured pulse train
pub struct BufferedReceiver<'a, SM> {
    sm: SM,
    buf: &'a [u32],
    i: usize,
    precalc_mult: u32,
}

impl<'a, SM: ReceiverSM> BufferedReceiver<'a, SM> {
    pub fn new(buf: &'a [u32], samplerate: u32) -> Self {
        Self {
            buf,
            i: 0,
            sm: SM::create(),
            precalc_mult: 1_000_000 / samplerate,
        }
    }
}

impl<'a, SM: ReceiverSM> Iterator for BufferedReceiver<'a, SM> {
    type Item = SM::Cmd;

    /// Get the next Command
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.i == self.buf.len() {
                break None;
            }

            let pos_edge = self.i & 0x1 == 0;
            let dt_us = self.buf[self.i] * self.precalc_mult;
            self.i += 1;

            let state: State = self.sm.event(pos_edge, dt_us).into();

            match state {
                State::Idle | State::Receiving => {
                    continue;
                }
                State::Done => {
                    let cmd = self.sm.command();
                    self.sm.reset();
                    break cmd;
                }
                State::Error(_) => {
                    self.sm.reset();
                    break None;
                }
            }
        }
    }
}

/// Receiver state machine
pub trait ReceiverSM {
    /// The Resulting Command Type
    type Cmd: Command;
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
