use crate::{Receiver, ReceiverState};

const BUF_LEN: usize = 128;

pub struct TraceReceiver {
    pub buffer: [u16; BUF_LEN],
    pub samplerate: u32,
    pub timeout: u16,
    pub prev_pinval: bool,
    pub prev_samplenum: u32,
    pub buffer_index: usize,

    pub state: ReceiverState<(), ()>,
}


impl Receiver for TraceReceiver {
    type Cmd = ();
    type Err = ();

    fn sample(&mut self, pinval: bool, samplenum: u32) -> ReceiverState<(), ()> {

        if !self.ready() {
            return self.state;
        }

        if self.state == ReceiverState::Receiving && self.delta(samplenum) > self.timeout {
            self.state = ReceiverState::Done(());
        }


        else if self.prev_pinval != pinval {
            // Change detected
            return self.sample_edge(pinval, samplenum);
        }

        self.state
    }

    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err> {

        if !self.ready() {
            return self.state;
        }

        let delta = self.delta(sampletime);

        if delta > self.timeout {
            // Set the receiver in disabled state but return the Done state
            self.state = ReceiverState::Done(());
        }

        self.state = ReceiverState::Receiving;  // Idle or Receiving doesn't really matter in this receiver
        self.prev_samplenum = sampletime;
        self.prev_pinval = rising;

        self.sample_edge_delta(rising, delta)
    }

    fn sample_edge_delta(&mut self, _rising: bool, sampledelta: u16) -> ReceiverState<Self::Cmd, Self::Err> {

        if !self.ready() {
            return self.state;
        }

        self.buffer[self.buffer_index] = sampledelta;
        self.buffer_index += 1;

        if self.buffer_index == BUF_LEN {
            self.state = ReceiverState::Done(());
        }

        self.state
    }

    fn reset(&mut self) {
        self.state = ReceiverState::Idle;
        self.prev_samplenum = 0;
        self.prev_pinval = false;
        self.buffer_index = 0;

        for i in 0..self.buffer.len() {
            self.buffer[i] = 0;
        }
    }

    fn disable(&mut self) {
        self.state = ReceiverState::Disabled;
    }
}

impl TraceReceiver {
    pub const fn new(samplerate: u32, timeout: u16) -> Self {
        Self {
            buffer: [0; BUF_LEN],
            samplerate,
            timeout,
            prev_pinval: false,
            prev_samplenum: 0,
            buffer_index: 0,
            state: ReceiverState::Idle
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
        &self.buffer[0..self.buffer_index]
    }
}
