use crate::Receiver;

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
    fn event(&mut self, ts: u32) -> Result<Option<TraceResult>, ()> {

        let t = if self.logdiffs {
            ts.wrapping_sub(self.ts_prev)
        } else {
            ts
        };

        self.data[self.ts_idx] = t;
        self.ts_prev = ts;

        self.ts_idx += 1;

        if self.ts_idx == BUF_LEN {
            Ok(Some(TraceResult { buf: self.data }))
        } else {
            Ok(None)
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
