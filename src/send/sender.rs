use crate::send::{PulsedataBuffer, ToPulsedata};

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

pub struct PulsedataSender {
    pub ptb: PulsedataBuffer,
    index: usize,
    pub(crate) state: State,
    ts_lastedge: u32,
}

impl PulsedataSender {
    pub fn new(samplerate: u32) -> Self {
        let ptb = PulsedataBuffer::with_samplerate(samplerate);
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
    pub fn load_command(&mut self, c: &impl ToPulsedata) {
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
