use crate::{Receiver, ReceiverState, ProtocolId};

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
    pub state: ReceiverState<()>,
}


impl Receiver for LoggingReceiver {
    type Cmd = ();
    const PROTOCOL_ID: ProtocolId = ProtocolId::Logging;


    fn sample(&mut self, pinval: bool, samplenum: u32) -> ReceiverState<()> {

        if !self.ready() {
            return self.state;
        }

        if self.prev_pinval != pinval {
            // Change detected
            return self.sample_edge(pinval, samplenum);
        }

        if self.state == ReceiverState::Receiving && self.delta(samplenum) > self.timeout {
            self.state = ReceiverState::Done(());
        }

        self.state
    }

    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd> {

        if !self.ready() {
            return self.state;
        }

        let delta = self.delta(sampletime);

        if delta > self.timeout {
            self.state = ReceiverState::Done(());
            return self.state;
        }

        self.state = ReceiverState::Receiving;
        self.prev_samplenum = sampletime;
        self.prev_pinval = rising;

        self.edges[self.n_edges] = delta;
        self.n_edges += 1;

        if self.n_edges == BUF_LEN {
            self.state = ReceiverState::Done(());
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

    fn disable(&mut self) {
        self.state = ReceiverState::Disabled;
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
        !(self.state == ReceiverState::Done(()) || self.state == ReceiverState::Disabled)
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
