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

    pub rc6_counter: u32,
    pub headerdata: u32,
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
            rc6_counter: 0,
            headerdata: 0,
        }
    }

    // How many RC6 base units is this pulse composed of
    pub fn rc6_units(&self, interval: u32) -> Option<u32> {

        for i in 1..=8 {
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
    Data(u32),
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

            let interval = timestamp.wrapping_sub(self.last);
            self.last = timestamp;
            // Debug:
            self.last_interval = interval;
            // Debug: Save the last state
            self.last_state = self.state;

            let rc6_units = self.rc6_units(interval);

            if let Some(units) = rc6_units {
                self.rc6_counter += units;
            }

            self.rc6_debug = rc6_units;

            let odd = self.rc6_counter & 1 == 1;

            let intern = match (self.state, rising, rc6_units) {
                (Idle,          FALLING,    _)      => Idle,
                (Idle,          RISING,     _)      => Leading,
                (Leading,       FALLING, Some(6))   => LeadingPaus,
                (Leading, _, _)                     => Idle,
                (LeadingPaus,   RISING,  Some(2))   => HeaderData(3),
                (LeadingPaus, _, _)                 => Idle,

                // Header Data 1 0 0 0
                (HeaderData(n), _, Some(_)) if odd => {
                    self.headerdata |= if rising {0} else {1} << n;
                    if n == 0 {
                        Trailing
                    } else {
                        HeaderData(n-1)
                    }
                },
                (HeaderData(n), _, Some(_)) => HeaderData(n),
                (HeaderData(_),         _,      None) => Idle,

                (Trailing, FALLING, Some(3))    => Data(15),
                (Trailing, RISING, Some(_))     => Data(15), //TODO
                (Trailing, FALLING, Some(1))    => Trailing,
                (Trailing, _, _) => Idle,

                (Data(0), _, Some(_)) if odd => {
                    self.data |= if rising {0} else {1} << 0;
                    Done
                },
                (Data(0), _, Some(_)) => Data(0),
                (Data(n), _, Some(_)) if odd => {
                    self.data |= if rising {0} else {1} << n;
                    Data(n-1)
                },
                (Data(n), _, Some(_)) => Data(n),
                (Data(_),      _,      None)   => Idle,

                (Done, _, _) => InternalState::Done,
            };

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
        self.headerdata = 0;
        self.rc6_counter = 0;
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

