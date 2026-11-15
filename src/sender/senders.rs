use crate::sender::{DEFAULT_BUFFER_SIZE, ProtocolEncoder, PulseBuffer};

/// Hardware independent sender, driven by a periodic tick
///
/// Load a command with [`load`](TickSender::load) and call
/// [`tick`](TickSender::tick) with a timestamp counting at `resolution` Hz.
/// The returned [`Status`] tells whether the carrier should be on.
pub struct TickSender<const S: usize = DEFAULT_BUFFER_SIZE> {
    ptb: PulseBuffer<S>,
    resolution: u32,
    pos: usize,
    status: Status,
    ts_lastedge: u32,
}

impl<const S: usize> TickSender<S> {
    /// Create a sender with timestamps counting at `resolution` Hz
    pub fn new(resolution: u32) -> Self {
        Self {
            ptb: PulseBuffer::new(),
            resolution,
            pos: 0,
            status: Status::Idle,
            ts_lastedge: 0,
        }
    }

    /// Timestamp resolution in Hz
    pub fn resolution(&self) -> u32 {
        self.resolution
    }

    /// Abort any transmission and clear the buffer
    pub fn reset(&mut self) {
        self.pos = 0;
        self.ts_lastedge = 0;
        self.status = Status::Idle;
        self.ptb.reset();
    }

    /// Load a command, replacing any transmission in progress
    ///
    /// Fails to compile if the buffer is too small for the protocol.
    pub fn load<Proto: ProtocolEncoder>(&mut self, cmd: &Proto::Cmd) {
        self.reset();
        self.ptb.load::<Proto>(cmd, self.resolution);
        // Busy until the pulse train has been sent
        if !self.ptb.buffer().is_empty() {
            self.status = Status::Transmit(false);
        }
    }

    /// True if no transmission is in progress
    pub fn is_idle(&self) -> bool {
        self.status == Status::Idle
    }

    /// Advance the sender to timestamp `ts`
    pub fn tick(&mut self, ts: u32) -> Status {
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

    /// The loaded pulse train in ticks, starting with a zero length entry
    pub fn buffer(&self) -> &[u32] {
        self.ptb.buffer()
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Sender state
pub enum Status {
    /// Sender is ready for transmitting
    #[default]
    Idle,
    /// Transmitting, `true` while the carrier is on
    Transmit(bool),
    /// Error
    Error,
}

/// A command was loaded while a transmission was in progress
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Busy;
