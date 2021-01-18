//! Transmitter state machine
//!

use core::convert::TryFrom;
use crate::PulseLengths;


#[derive(Debug, PartialEq, Copy, Clone)]
/// Sender state
pub enum State {
    /// Sender is ready for transmitting
    Idle,
    /// Transmitting
    Transmit(bool),
    /// Error
    Error,
}

pub struct PulseSender {
    pub ptb: PulseBuffer,
    index: usize,
    pub(crate) state: State,
    ts_lastedge: u32,
}

impl PulseSender {
    pub fn new(samplerate: u32) -> Self {
        let ptb = PulseBuffer::with_samplerate(samplerate);
        Self {
            ptb,
            index: 0,
            state: State::Idle,
            ts_lastedge: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.ts_lastedge = 0;
        self.state = State::Idle;
        self.ptb.reset();
    }

    /// Load command into internal buffer
    pub fn load_command(&mut self, c: &impl PulseLengths) {
        self.reset();
        self.ptb.load(c);
    }

    pub fn tick(&mut self, ts: u32) -> State {
        if let Some(dist) = self.ptb.get(self.index) {
            let delta_ts = ts.wrapping_sub(self.ts_lastedge);
            if delta_ts >= u32::from(dist) {
                let newstate = match self.state {
                    State::Idle | State::Transmit(false) => State::Transmit(true),
                    _ => State::Transmit(false),
                };

                self.state = newstate;
                self.index += 1;
                self.ts_lastedge = ts;
            }
        } else {
            self.state = State::Idle;
        }

        self.state
    }

    pub fn buffer(&self) -> &[u16] {
        self.ptb.buffer()
    }
}

pub struct PulseBuffer {
    pub buf: [u16; 96],
    pub len: usize,
    pub scaler: u16,
}

impl PulseBuffer {
    pub fn new() -> Self {
        Self {
            buf: [0; 96],
            len: 0,
            scaler: 1,
        }
    }

    pub fn reset(&mut self) {
        self.len = 0;
    }

    pub fn with_samplerate(samplerate: u32) -> Self {
        Self {
            buf: [0; 96],
            len: 0,
            scaler: u16::try_from(1000 / (samplerate / 1000)).unwrap(),
        }
    }

    pub fn load(&mut self, c: &impl PulseLengths) {
        let len = c.encode(&mut self.buf);
        self.len = len;

        // Apply the scaling on the buf
        for elem in &mut self.buf[0..len] {
            *elem /= self.scaler;
        }
    }

    pub fn get(&self, index: usize) -> Option<u16> {
        self.buf.get(index).cloned()
    }

    pub fn buffer(&self) -> &[u16] {
        &self.buf[..self.len]
    }
}

impl<'a> IntoIterator for &'a PulseBuffer {
    type Item = u16;
    type IntoIter = PulseIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PulseIterator {
            pulses: &self.buf[0..self.len],
            index: 0,
        }
    }
}

pub struct PulseIterator<'a> {
    pulses: &'a [u16],
    index: usize,
}

impl<'a> Iterator for PulseIterator<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.pulses.len() {
            None
        } else {
            let r = self.pulses[self.index];
            self.index += 1;
            Some(r)
        }
    }
}
