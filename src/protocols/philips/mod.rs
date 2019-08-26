use crate::{
    Receiver, ReceiverState,
};
use core::ops::Range;

// Timing (us)
const HEADER_LEADING: u32 = 2666;
const HEADER_TRAILING: u32 = 1350;
const DATA: u32 = 444;


pub struct PhilipsReceiver {
    samplerate: u32,
    state: InternalState,
    last: u32,
    pinval: bool,
}

enum PulseType {
    HeaderLeading,
    HeaderTrailing,
    DataZero,
    DataOne,

    Multiple(u32),

    None,
}


impl PhilipsReceiver {

    pub fn new(samplerate: u32) -> Self {
        Self {
            samplerate,
            last: 0,
            state: InternalState::Idle,
            pinval: false,
        }
    }

    // How many RC6 base units is this pulse composed of
    fn rc6_units(&self, interval: u32) -> Some(u32) {

        for i in 1..=6 {
            if rc6_multiplier(self.samplerate, i).contains(&interval) {
                Some(i)
            }
        }

        None
    }
}



enum InternalState {
    Idle,
    Leading,
    Header,
    Trailing,
    Data,
    Done,
}

impl Receiver for PhilipsReceiver {
    type Command = ();
    type ReceiveError = ();

    fn event(&mut self, rising: bool, timestamp: u32) -> ReceiverState<Self::Command, Self::ReceiveError> {

        if self.pinval != rising {

            let len = self.last.wrapping_sub(timestamp);

            if let Some(units) = self.rc6_units(len) {

            }

            let intern = match (self.state, rising) {
                (InternalState::Idle, false) => {
                    //TODO: Unreachable
                    InternalState::Idle
                },
                (InternalState::Idle, true) => {
                    self.last = timestamp;
                    InternalState::Leading
                },
                (InternalState::Leading, false) => {
                    // Leading header
                    //self.is_leader(delta)
                    InternalState::Header
                },
                (InternalState::Leading, _) => InternalState::Header,
                (InternalState::Header, _) => InternalState::Trailing,
                (InternalState::Trailing, _) => InternalState::Data,
                (InternalState::Data, _) => InternalState::Done,
                (InternalState::Done, _) => InternalState::Done,
            };

            self.state = intern;
        }

        match self.state {
            _ => ReceiverState::Receiving
        }
    }

    fn reset(&mut self) {
        unimplemented!()
    }

    fn disable(&mut self) {
        unimplemented!()
    }
}

const fn rc6_multiplier(samplerate: u32, multiplier: u32) -> Range<u32> {
    let base = (samplerate * 444 * multiplier) / 1000_000;
    range(base, 5)
}

const fn range(len: u32, percent: u32) -> Range<u32> {
    let tol = (len * percent) / 100;

     Range {
        start: len - tol,
        end: len + tol,
    }
}

