use crate::{
    Receiver, ReceiverState,
};
use core::ops::Range;


pub struct PhilipsCommand {
    pub data: u32,
}

pub struct PhilipsReceiver {
    samplerate: u32,
    pub state: InternalState,
    pub rc6_debug: Option<u32>,
    pub last: u32,
    pub last_interval: u32,
    pub last_state: InternalState,
    pinval: bool,
    pub data: u32,
    pub data_idx: usize,
}

impl PhilipsReceiver {

    pub fn new(samplerate: u32) -> Self {
        Self {
            samplerate,
            last: 0,
            last_interval: 0,
            state: InternalState::Idle,
            last_state: InternalState::Idle,
            rc6_debug: None,
            pinval: false,
            data: 0,
            data_idx: 0
        }
    }

    // How many RC6 base units is this pulse composed of
    pub fn rc6_units(&self, interval: u32) -> Option<u32> {

        for i in 1..=6 {
            if rc6_multiplier(self.samplerate, i).contains(&interval) {
                return Some(i);
            }
        }

        None
    }
}



#[derive(Clone, Copy, Debug)]
pub enum InternalState {
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
    type Command = PhilipsCommand;
    type ReceiveError = ();

    fn event(&mut self, rising: bool, timestamp: u32) -> ReceiverState<Self::Command, Self::ReceiveError> {

        if self.pinval != rising {
            use InternalState::*;

            //TODO: Check if alright in all situations
            let interval = timestamp.wrapping_sub(self.last);
            self.last = timestamp;
            self.last_interval = interval;
            self.last_state = self.state;

            let rc6_units = self.rc6_units(interval);

            self.rc6_debug = rc6_units;

            let intern = match (self.state, rising, rc6_units) {
                (Idle,          FALLING,    _)      => Idle,        // Unrechable
                (Idle,          RISING,     _)      => Leading,
                (Leading,       FALLING, Some(6))   => LeadingPaus,
                (Leading, _, _)                     => Idle,
                (LeadingPaus,   RISING,  Some(2))   => HeaderData(0),
                (LeadingPaus, _, _)                 => Idle,

                // Header Data 1 0 0 0
                (HeaderData(0), FALLING, Some(1))   => HeaderData(1),
                (HeaderData(1), FALLING, Some(1))   => HeaderData(1),
                (HeaderData(1), RISING,  Some(2))   => HeaderData(2),
                (HeaderData(2), FALLING, Some(1))   => HeaderData(2),
                (HeaderData(2), RISING,  Some(1))   => HeaderData(3),
                (HeaderData(3), FALLING, Some(1))   => HeaderData(3),
                (HeaderData(3), RISING,  Some(1))   => HeaderData(4),
                (HeaderData(4), FALLING, Some(3))   => Trailing,
                (HeaderData(_), _, _)               => Idle,

                (Trailing, RISING, Some(3)) => Data(false),
                (Trailing, RISING, Some(2)) => Data(true),
                //(Trailing, FALLING, Some(3)) => Data(false),
                (Trailing, _, _) => Idle,

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
            InternalState::Idle => ReceiverState::Idle,
            InternalState::Done => ReceiverState::Done(PhilipsCommand {data: self.data}),
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
    range(base, 10)
}

const fn range(len: u32, percent: u32) -> Range<u32> {
    let tol = (len * percent) / 100;

     Range {
        start: len - tol,
        end: len + tol + 2,
    }
}

