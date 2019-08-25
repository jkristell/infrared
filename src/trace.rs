use crate::{Receiver, ReceiverState};

const BUF_LEN: usize = 128;

pub struct TraceReceiver {
    pub data: [u32; BUF_LEN],
    pub samplerate: u32,
    pub prev_pinval: bool,
    pub st_prev: u32,
    pub event_id: usize,
    pub enabled: bool,
}

pub struct TraceResult {
    pub buf: [u32; BUF_LEN],
    pub buf_len: usize,
}

fn pin_changed(prev: bool, pinval: bool) -> Option<bool> {
    match (pinval, prev) {
        (false, true) => Some(true),
        (true, false) => Some(false),
        _ => None,
    }
}


const TIMEOUT: u32 = 1000;

impl Receiver for TraceReceiver {
    type Command = TraceResult;
    type ReceiveError = ();

    fn event(&mut self, pinvalue: bool, st: u32) -> ReceiverState<TraceResult, ()> {

        if !self.enabled {
            return ReceiverState::Disabled;
        }

        // Number of  samples since last pin value change
        let sampledelta = match self.st_prev {
            0 => 0,
            _ => st.wrapping_sub(self.st_prev),
        };

        if sampledelta > TIMEOUT {
            // Set the receiver in disabled state but return the Done state
            self.enabled = false;
            return ReceiverState::Done(TraceResult {
                buf: self.data,
                buf_len: self.event_id,
            });
        }

        if let Some(_rising) = pin_changed(self.prev_pinval, pinvalue) {
            // Change detected
            self.data[self.event_id] = sampledelta;
            self.event_id += 1;
            self.st_prev = st;

            self.prev_pinval = pinvalue;

            if self.event_id == BUF_LEN {
                ReceiverState::Done(TraceResult {
                    buf: self.data,
                    buf_len: self.event_id,
                })
            } else {
                ReceiverState::Receiving
            }
        } else {
            ReceiverState::Receiving
        }
    }

    fn reset(&mut self) {
        self.st_prev = 0;
        self.event_id = 0;
        self.prev_pinval = false;
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false;
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
            enabled: true,
        }
    }
}
