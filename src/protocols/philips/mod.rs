use crate::{
    Receiver, ReceiverState,
};
use core::ops::Range;



pub struct PhilipsReceiver {
    samplerate: u32,
    state: InternalState,
    last: u32,
    pinval: bool,
    pub data: u32,
    pub data_idx: usize,
}

impl PhilipsReceiver {

    pub fn new(samplerate: u32) -> Self {
        Self {
            samplerate,
            last: 0,
            state: InternalState::Idle,
            pinval: false,
            data: 0,
            data_idx: 0
        }
    }

    // How many RC6 base units is this pulse composed of
    fn rc6_units(&self, interval: u32) -> Option<u32> {

        for i in 1..=6 {
            if rc6_multiplier(self.samplerate, i).contains(&interval) {
                return Some(i);
            }
        }

        None
    }
}



#[derive(Clone, Copy)]
enum InternalState {
    Idle,
    Leading,
    LeadingPaus,
    HeaderData(u32),
    Trailing,
    Data(bool),
    Done,
}

const RISING: bool = true;
const FALLING: bool = false;

impl Receiver for PhilipsReceiver {
    type Command = ();
    type ReceiveError = ();

    fn event(&mut self, rising: bool, timestamp: u32) -> ReceiverState<Self::Command, Self::ReceiveError> {

        if self.pinval != rising {
            use InternalState::*;

            //TODO: Check if alright in all situations
            let interval = self.last.wrapping_sub(timestamp);
            self.last = timestamp;

            let rc6_units = self.rc6_units(interval);

            let intern = match (self.state, rising, rc6_units) {
                (Idle, false, _)                => Idle,        //TODO: Unreachable
                (Idle, true, _)                 => Leading,
                (Leading, false, Some(6))       => LeadingPaus,
                (Leading, _, _)                 => Idle,
                (LeadingPaus, true, Some(2))    => HeaderData(0),
                (LeadingPaus, _, _)             => Idle,

                // Header Data 1 0 0 0
                (HeaderData(0), RISING,  Some(1))   => HeaderData(1),
                (HeaderData(1), FALLING, Some(1))   => HeaderData(1),
                (HeaderData(1), RISING,  Some(2))   => HeaderData(2),
                (HeaderData(2), FALLING, Some(1))   => HeaderData(2),
                (HeaderData(2), RISING,  Some(1))   => HeaderData(3),
                (HeaderData(3), FALLING, Some(1))   => HeaderData(3),
                (HeaderData(3), RISING,  Some(1))   => HeaderData(4),
                (HeaderData(4), FALLING, Some(1))   => Trailing,
                (HeaderData(_), _, _)               => Idle,

                (Trailing, _, _) => InternalState::Data(false),

                (Data(_), RISING, Some(1)) => Data(false),
                (Data(_), FALLING, Some(2)) => Data(true),
                (Data(_), RISING, Some(2)) => Data(false),
                (Data(_), FALLING, Some(1)) => Data(true),


                (Data(_), _, _) => InternalState::Done,

                (Done, _, _) => InternalState::Done,
            };

            if let Data(v) = intern {
                if v {
                    self.data |= 1 << self.data_idx;
                }
                self.data_idx += 1;
            }

            self.pinval = rising;
            self.state = intern;
        }

        match self.state {
            _ => ReceiverState::Receiving
        }
    }

    fn reset(&mut self) {

        self.state = InternalState::Idle;
        self.pinval = false;
        self.data = 0;
        self.data_idx = 0;
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

