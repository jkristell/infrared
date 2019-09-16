use crate::{
    Receiver, ReceiverState,
};
use core::ops::Range;

#[derive(Debug)]
pub struct PhilipsCommand {
    pub addr: u8,
    pub cmd: u8,
    pub repeat: bool,
}

impl PhilipsCommand {
    pub fn new(data: u32, repeat: bool) -> Self {
        let addr = (data >> 8) as u8;
        let cmd = (data & 0xFF) as u8;

        Self {
            addr,
            cmd,
            repeat,
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub enum PhilipsError {
    Header(u32),
    Data(u32),
    Rc6Version(u32),
}


pub struct PhilipsReceiver {
    samplerate: u32,
    state: InternalState,
    pinval: bool,
    data: u32,
    headerdata: u32,
    repeat: bool,
    last: u32,
    pub rc6_counter: u32,

    pub last_interval: u32,
    pub last_state: InternalState,
}


impl PhilipsReceiver {

    pub fn new(samplerate: u32) -> Self {
        Self {
            samplerate,
            last: 0,
            last_interval: 0,
            state: InternalState::Idle,
            last_state: InternalState::Idle,
            pinval: false,
            data: 0,
            rc6_counter: 0,
            headerdata: 0,
            repeat: false,
        }
    }

    fn rc6_units(&self, interval: u32) -> Option<u32> {
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
    Error(PhilipsError),
}

const RISING: bool = true;
const FALLING: bool = false;

impl Receiver for PhilipsReceiver {
    type Command = PhilipsCommand;
    type ReceiveError = PhilipsError;

    fn event(&mut self, rising: bool, timestamp: u32) -> ReceiverState<Self::Command, Self::ReceiveError> {

        if self.pinval != rising {
            use InternalState::*;

            let interval = timestamp.wrapping_sub(self.last);
            self.last = timestamp;
            // Debug:
            self.last_interval = interval;
            self.last_state = self.state;

            // Nbr of rc6_units since last pin edge
            let n_units = self.rc6_units(interval);

            if let Some(units) = n_units {
                self.rc6_counter += units;
            }

            let odd = self.rc6_counter & 1 == 1;

            let next = match (self.state, rising, n_units) {
                (Idle,          FALLING,    _)      => Idle,
                (Idle,          RISING,     _)      => Leading,
                (Leading,       FALLING, Some(6))   => LeadingPaus,
                (Leading, _, _)                     => Idle,
                (LeadingPaus,   RISING,  Some(2))   => HeaderData(3),
                (LeadingPaus, _, _)                 => Idle,

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

                (Trailing, FALLING, Some(3))    => {
                    self.repeat = false;
                    Data(15)
                },
                (Trailing, RISING, Some(_))     => {
                    self.repeat = true;
                    Data(15)
                },
                (Trailing, FALLING, Some(1))    => Trailing,
                (Trailing, _, _) => Idle,

                (Data(0), _, Some(_)) if odd => {
                    self.data |= if rising {0} else {1} ;
                    Done
                },
                (Data(0), _, Some(_)) => Data(0),
                (Data(n), _, Some(_)) if odd => {
                    self.data |= if rising {0} else {1} << n;
                    Data(n-1)
                },
                (Data(n), _, Some(_)) => Data(n),
                (Data(_),      _,      None)   => Error(PhilipsError::Data(interval)),     // Data Error

                (Done, _, _) => InternalState::Done,
                (Error(err), _, _) => InternalState::Error(err),
            };

            self.pinval = rising;
            self.state = next;
        }

        match self.state {
            InternalState::Idle => ReceiverState::Idle,
            InternalState::Done => {
                let cmd = PhilipsCommand::new(self.data, self.repeat);
                ReceiverState::Done(cmd)
            },
            InternalState::Error(err) => ReceiverState::Err(err),
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
        start: len - tol - 2,
        end: len + tol + 4,
    }
}

