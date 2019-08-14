use crate::{Receiver, ReceiverState};

const BUF_LEN: usize = 32;

pub struct TraceReceiver {
    pub data: [u32; BUF_LEN],
    prev_pinval: bool,
    pub st_prev: u32,
    pub event_id: usize,
    state: ReceiverState<TraceResult, ()>,
}

#[derive(Debug, PartialEq)]
pub struct TraceResult {
    pub buf: [u32; BUF_LEN],
}

fn pin_xor(prev: bool, pinval: bool) -> Option<bool> {
    match (pinval, prev) {
        (false, true) => Some(true),
        (true, false) => Some(false),
        _ => None,
    }
}


const TIMEOUT: u32 = 10_000;

impl Receiver for TraceReceiver {
    type Command = TraceResult;
    type ReceiveError = ();

    // st - sample time
    fn event(&mut self, pinvalue: bool, st: u32) -> ReceiverState<TraceResult, ()> {

        // Number of  samples since last pin value change
        let delta = match self.st_prev {
            0 => 0,
            _ => st.wrapping_sub(self.st_prev),
        };

        match self.state {
            ReceiverState::Disabled => return ReceiverState::Disabled,
            ReceiverState::Receiving if delta > TIMEOUT => return ReceiverState::Done(TraceResult { buf: self.data }),
            _ => (),
        };


        if let Some(_rising) = pin_xor(self.prev_pinval, pinvalue) {
            // Change detected
            self.data[self.event_id] = delta;
            self.event_id += 1;
            self.st_prev = st;

            self.prev_pinval = pinvalue;

            if self.event_id == BUF_LEN {
                ReceiverState::Done(TraceResult { buf: self.data })
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
        self.state = ReceiverState::Idle;
    }

    fn disable(&mut self) {
        self.state = ReceiverState::Disabled;
    }
}

impl TraceReceiver {
    pub const fn new(samplerate: u32) -> Self {

        // Define timeout to be 1 s
        let _timeout = (1 * 1000) / (samplerate * 1000);

        Self {
            data: [0; BUF_LEN],
            prev_pinval: false,
            st_prev: 0,
            event_id: 0,
            state: ReceiverState::Idle
        }
    }
}
