use crate::prelude::*;
use crate::receiver::{ReceiverState, ReceiverStateMachine};

const BUF_LEN: usize = 128;

/// Receiver that doesn't do any decoding of the incoming signal
/// Instead it saves the distance between the edges for later processing
pub struct LoggingReceiver {
    /// Samplerate
    pub samplerate: u32,
    /// Timemout
    pub timeout: u16,
    /// Saved edges
    pub edges: [u16; BUF_LEN],
    /// Number of edges in edges
    pub n_edges: usize,
    /// Prev pin value
    pub prev_pinval: bool,
    /// Samplenum with pin change
    pub prev_samplenum: u32,
    /// Our state
    pub state: ReceiverState<DummyCommand>,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct DummyCommand;

impl Command for DummyCommand {
    fn construct(_addr: u16, _cmd: u8) -> Self {
        DummyCommand
    }

    fn address(&self) -> u16 {
        0
    }

    fn command(&self) -> u8 {
        0
    }
}

impl ReceiverStateMachine for LoggingReceiver {
    const ID: ProtocolId = ProtocolId::Logging;
    type Cmd = DummyCommand;

    fn for_samplerate(samplerate: u32) -> Self {
        Self::new(samplerate, (samplerate / 1000) as u16)
    }

    fn event(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd> {
        if !self.ready() {
            return self.state;
        }

        let delta = self.delta(sampletime);

        if delta > self.timeout {
            self.state = ReceiverState::Done(DummyCommand);
            return self.state;
        }

        self.state = ReceiverState::Receiving;
        self.prev_samplenum = sampletime;
        self.prev_pinval = rising;

        self.edges[self.n_edges] = delta;
        self.n_edges += 1;

        if self.n_edges == BUF_LEN {
            self.state = ReceiverState::Done(DummyCommand);
        }

        self.state
    }

    fn reset(&mut self) {
        self.state = ReceiverState::Idle;
        self.prev_samplenum = 0;
        self.prev_pinval = false;
        self.n_edges = 0;

        for i in 0..self.edges.len() {
            self.edges[i] = 0;
        }
    }
}

impl LoggingReceiver {
    pub const fn new(samplerate: u32, timeout: u16) -> Self {
        Self {
            edges: [0; BUF_LEN],
            samplerate,
            timeout,
            prev_pinval: false,
            prev_samplenum: 0,
            n_edges: 0,
            state: ReceiverState::Receiving,
        }
    }

    fn ready(&self) -> bool {
        !(self.state == ReceiverState::Done(DummyCommand) || self.state == ReceiverState::Disabled)
    }

    pub fn delta(&self, ts: u32) -> u16 {
        if self.prev_samplenum == 0 {
            return 0;
        }

        ts.wrapping_sub(self.prev_samplenum) as u16
    }

    pub fn data(&self) -> &[u16] {
        &self.edges[0..self.n_edges]
    }
}
