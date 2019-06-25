use crate::{Receiver, State};

const BUF_LEN: usize = 32;

pub struct TraceReceiver {
    pub data: [u32; BUF_LEN],
    logdiffs: bool,
    pub ts_prev: u32,
    pub ts_idx: usize,
}

#[derive(Debug)]
pub struct TraceResult {
    pub buf: [u32; BUF_LEN],
}

impl Receiver<TraceResult, ()> for TraceReceiver {
    fn event(&mut self, _rising: bool, ts: u32) -> State<TraceResult, ()>  {

        let t = if self.logdiffs {
            ts.wrapping_sub(self.ts_prev)
        } else {
            ts
        };

        self.data[self.ts_idx] = t;
        self.ts_idx += 1;
        self.ts_prev = ts;

        if self.ts_idx == BUF_LEN {
            State::Done(TraceResult { buf: self.data })
        } else {
            State::InProgress
        }
    }

    fn reset(&mut self) {
        self.ts_prev = 0;
        self.ts_idx = 0;
    }

    fn disable(&mut self) {
    }
}

impl TraceReceiver {
    pub const fn new(logdiffs: bool) -> Self {
        Self {
            data: [0; BUF_LEN],
            ts_prev: 0,
            ts_idx: 0,
            logdiffs,
        }
    }
}
