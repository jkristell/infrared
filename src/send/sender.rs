use crate::send::{InfraredSender, PulsedataBuffer, };

#[derive(Debug, PartialEq, Copy, Clone)]
/// Sender state
pub enum Status {
    /// Sender is ready for transmitting
    Idle,
    /// Transmitting
    Transmit(bool),
    /// Error
    Error,
}

pub struct PulsedataSender<Protocol: InfraredSender> {
    pub ptb: PulsedataBuffer<Protocol>,
    index: usize,
    pub(crate) status: Status,
    ts_lastedge: u32,
}

impl<Proto: InfraredSender> PulsedataSender<Proto> {
    pub fn new(samplerate: u32) -> Self {
        let ptb = PulsedataBuffer::with_samplerate(samplerate);
        Self {
            ptb,
            index: 0,
            status: Status::Idle,
            ts_lastedge: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.ts_lastedge = 0;
        self.status = Status::Idle;
        self.ptb.reset();
    }

    /// Load command into internal buffer
    pub fn load_command(&mut self, c: &Proto::Cmd) {
        self.reset();
        self.ptb.load(c);
    }

    pub fn tick(&mut self, ts: u32) -> Status {
        if let Some(dist) = self.ptb.get(self.index) {
            let delta_ts = ts.wrapping_sub(self.ts_lastedge);
            if delta_ts >= u32::from(dist) {
                let newstate = match self.status {
                    Status::Idle | Status::Transmit(false) => Status::Transmit(true),
                    _ => Status::Transmit(false),
                };

                self.status = newstate;
                self.index += 1;
                self.ts_lastedge = ts;
            }
        } else {
            self.status = Status::Idle;
        }

        self.status
    }

    pub fn buffer(&self) -> &[u16] {
        self.ptb.buffer()
    }
}
