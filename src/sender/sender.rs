use crate::sender::{ProtocolEncoder, PulsedataBuffer};

#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Sender state
pub enum Status {
    /// Sender is ready for transmitting
    Idle,
    /// Transmitting
    Transmit(bool),
    /// Error
    Error,
}

pub struct PulsedataSender<const S: usize> {
    pub(crate) ptb: PulsedataBuffer<S>,
    pos: usize,
    pub(crate) status: Status,
    ts_lastedge: usize,
}

impl<const S: usize> PulsedataSender<S> {
    pub fn new() -> Self {
        let ptb = PulsedataBuffer::new();
        Self {
            ptb,
            pos: 0,
            status: Status::Idle,
            ts_lastedge: 0,
        }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
        self.ts_lastedge = 0;
        self.status = Status::Idle;
        self.ptb.reset();
    }

    /// Load command into internal buffer
    pub fn load_command<Proto: ProtocolEncoder<F>, const F: usize>(&mut self, c: &Proto::Cmd) {
        self.reset();
        self.ptb.load::<Proto, F>(c);
    }

    pub fn tick(&mut self, ts: usize) -> Status {
        if let Some(dist) = self.ptb.get(self.pos) {
            let delta_ts = ts.wrapping_sub(self.ts_lastedge);
            if delta_ts >= dist {
                let newstate = match self.status {
                    Status::Idle | Status::Transmit(false) => Status::Transmit(true),
                    _ => Status::Transmit(false),
                };

                self.status = newstate;
                self.pos += 1;
                self.ts_lastedge = ts;
            }
        } else {
            self.status = Status::Idle;
        }

        self.status
    }

    pub fn buffer(&self) -> &[usize] {
        self.ptb.buffer()
    }
}
