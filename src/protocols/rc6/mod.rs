use core::ops::Range;
use crate::{Receiver, ReceiverState};
#[cfg(feature="protocol-dev")]
use crate::ReceiverDebug;

#[derive(Debug)]
pub struct Rc6Command {
    pub addr: u8,
    pub cmd: u8,
    pub toggle: bool,
}

impl Rc6Command {
    pub fn new(data: u32, toggle: bool) -> Self {
        let addr = (data >> 8) as u8;
        let cmd = (data & 0xFF) as u8;
        Self {addr, cmd, toggle}
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc6Error {
    Header(u16),
    Data(u16),
    Rc6Version(u16),
}

pub struct Rc6Receiver {
    samplerate: u32,
    state: Rc6State,
    pub pinval: bool,
    data: u32,
    headerdata: u32,
    toggle: bool,
    pub last: u32,
    pub rc6_counter: u32,

    #[cfg(feature="protocol-dev")]
    pub debug: ReceiverDebug<Rc6State, Option<u32>>,
}

impl Rc6Receiver {
    pub fn new(samplerate: u32) -> Self {
        Self {
            samplerate,
            last: 0,
            state: Rc6State::Idle,
            pinval: false,
            data: 0,
            rc6_counter: 0,
            headerdata: 0,
            toggle: false,
            #[cfg(feature="protocol-dev")]
            debug: ReceiverDebug {
                state: Rc6State::Idle,
                state_new: Rc6State::Idle,
                delta: 0,
                extra: None,
            }
        }
    }

    fn interval_to_units(&self, interval: u16) -> Option<u32> {

        let interval = u32::from(interval);

        for i in 1..=8 {
            if rc6_multiplier(self.samplerate, i).contains(&interval) {
                return Some(i);
            }
        }
        None
    }

    // Time since last edge
    fn delta(&self, sampletime: u32) -> u16 {
        let delta = sampletime.wrapping_sub(self.last);
        if delta >= core::u16::MAX.into() {
            return 0;
        }
        delta as u16
    }

    fn receiver_state(&self) -> Rc6Result {
        use ReceiverState::*;
        match self.state {
            Rc6State::Idle => Idle,
            Rc6State::Done => Done(Rc6Command::new(self.data, self.toggle)),
            Rc6State::Error(err) => Error(err),
            _ => Receiving
        }
    }
}

type Rc6Result = ReceiverState<Rc6Command, Rc6Error>;

#[derive(Clone, Copy, Debug)]
pub enum Rc6State {
    Idle,
    Leading,
    LeadingPaus,
    HeaderData(u32),
    Trailing,
    Data(u32),
    Done,
    Error(Rc6Error),
}

const RISING: bool = true;
const FALLING: bool = false;

impl Receiver for Rc6Receiver {
    type Cmd = Rc6Command;
    type Err = Rc6Error;

    fn sample(&mut self, pinval: bool, timestamp: u32) -> Rc6Result {

        if self.pinval != pinval {
            return self.sample_edge(pinval, timestamp);
        }

        self.receiver_state()
    }

    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> Rc6Result {
        use Rc6State::*;

        let delta = self.delta(sampletime);
        self.last = sampletime;
        self.pinval = rising;

        // Number of rc6 units since last pin edge
        let n_units = self.interval_to_units(delta);

        if let Some(units) = n_units {
            self.rc6_counter += units;
        }

        let odd = self.rc6_counter & 1 == 1;

        let newstate = match (self.state, rising, n_units) {
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
            (HeaderData(n), _, Some(_))     => HeaderData(n),
            (HeaderData(_), _, None)        => Idle,

            (Trailing, FALLING, Some(3))    => {
                self.toggle = false;
                Data(15)
            },
            (Trailing, RISING, Some(2))     => {
                self.toggle = true;
                Data(15)
            },
            (Trailing, FALLING, Some(1))    => Trailing,
            (Trailing, _, _) => Idle,

            (Data(0), _, Some(_)) if odd    => {
                self.data |= if rising {0} else {1};
                Done
            },
            (Data(0), _, Some(_)) => Data(0),
            (Data(n), _, Some(_)) if odd    => {
                self.data |= if rising {0} else {1} << n;
                Data(n-1)
            },
            (Data(n), _, Some(_)) => Data(n),
            (Data(_),      _,      None)    => Error(Rc6Error::Data(delta)),

            (Done, _, _)        => Rc6State::Done,
            (Error(err), _, _)  => Rc6State::Error(err),
        };

        #[cfg(feature="protocol-dev")]
        {
            self.debug.state = self.state;
            self.debug.state_new = newstate;
            self.debug.delta = delta;
            self.debug.extra = n_units;
        }

        self.state = newstate;
        self.receiver_state()
    }

    fn reset(&mut self) {
        self.state = Rc6State::Idle;
        self.pinval = false;
        self.data = 0;
        self.last = 0;
        self.headerdata = 0;
        self.rc6_counter = 0;
    }

    fn disable(&mut self) {
        unimplemented!()
    }
}

pub const fn rc6_multiplier(samplerate: u32, multiplier: u32) -> Range<u32> {
    let base = (samplerate * 444 * multiplier) / 1_000_000;
    range(base, 10)
}

const fn range(len: u32, percent: u32) -> Range<u32> {
    let tol = (len * percent) / 100;

     Range {
        start: len - tol - 2,
        end: len + tol + 4,
    }
}

