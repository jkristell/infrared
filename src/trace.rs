use crate::{Receiver, ReceiverState};

const BUF_LEN: usize = 128;

pub struct TraceReceiver {
    pub data: [u16; BUF_LEN],
    pub samplerate: u32,
    pub prev_pinval: bool,
    pub st_prev: u32,
    pub event_id: usize,

    state: ReceiverState<TraceResult, ()>,
}

#[derive(Clone, Copy)]
pub struct TraceResult {
    pub buf: [u16; BUF_LEN],
    pub buf_len: usize,
}

//TODO: Make dependent on samplerate
const TIMEOUT: u16 = 1000;


impl Receiver for TraceReceiver {
    type Command = TraceResult;
    type ReceiveError = ();

    fn event(&mut self, pinvalue: bool, st: u32) -> ReceiverState<TraceResult, ()> {

        match self.state {
            ReceiverState::Disabled => return ReceiverState::Disabled,
            _ => (),
        };

        let delta = self.delta(st);

        if delta > TIMEOUT {
            // Set the receiver in disabled state but return the Done state
            self.state = ReceiverState::Idle;
            return ReceiverState::Done(TraceResult {
                buf: self.data,
                buf_len: self.event_id,
            });
        }

        if self.prev_pinval != pinvalue {
            // Change detected
            self.state = ReceiverState::Receiving;

            self.data[self.event_id] = delta;
            self.event_id += 1;

            self.st_prev = st;
            self.prev_pinval = pinvalue;

            if self.event_id == BUF_LEN {
                self.state = ReceiverState::Disabled;

                let mut buf = [0; 128];
                for i in 0..self.data.len() {
                    buf[i] = self.data[i];
                }

                return ReceiverState::Done(TraceResult {
                    buf,
                    buf_len: self.event_id,
                })
            }
        }

        match self.state {
            ReceiverState::Disabled => ReceiverState::Disabled,
            ReceiverState::Idle => ReceiverState::Idle,
            _ => ReceiverState::Receiving,
        }
    }

    fn reset(&mut self) {
        self.st_prev = 0;
        self.event_id = 0;
        self.prev_pinval = false;
        self.state = ReceiverState::Idle;

        for i in 0..self.data.len() {
            self.data[i] = 0;
        }
    }

    fn disable(&mut self) {
        self.state = ReceiverState::Disabled;
    }
}

impl TraceReceiver {
    pub const fn new(samplerate: u32) -> Self {

        Self {
            data: [0; BUF_LEN],
            samplerate,
            prev_pinval: false,
            st_prev: 0,
            event_id: 0,
            state: ReceiverState::Idle
        }
    }

    pub fn delta(&self, ts: u32) -> u16 {
        if self.st_prev == 0 {
            return 0;
        }

        ts.wrapping_sub(self.st_prev) as u16
    }
}
