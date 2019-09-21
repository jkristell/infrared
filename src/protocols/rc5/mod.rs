use core::ops::Range;
use crate::{Receiver, ReceiverState};

#[derive(Debug)]
pub struct Rc5Command {
    pub addr: u8,
    pub cmd: u8,
    pub start: u8,
    pub toggle: u8,
}

impl Rc5Command {
    pub fn new(data: u16) -> Self {

        //                   SS_TAAA_AACC_CCCC
        let addr_mask = 0b_0000_0111_1100_0000;
        let cmd_mask  = 0b_0000_0000_0011_1111;

        let addr = ((data & addr_mask) >> 6) as u8;
        let cmd = (data & cmd_mask) as u8;
        let start = (data >> 12) as u8;
        let toggle = ((data >> 11) & 1) as u8;

        Self {
            addr,
            cmd,
            start,
            toggle
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc5Error {
    Header(u32),
    Data(u32),
    Rc5Version(u32),
}

pub struct Rc5Receiver {
    samplerate: u32,
    state: InternalState,
    pinval: bool,
    data: u16,
    last: u32,
    pub rc5_counter: u32,

    pub last_interval: u32,
    pub last_state: InternalState,
}

impl Rc5Receiver {
    pub fn new(samplerate: u32) -> Self {
        Self {
            samplerate,
            last: 0,
            last_interval: 0,
            state: InternalState::Idle,
            last_state: InternalState::Idle,
            pinval: false,
            data: 0,
            rc5_counter: 0,
        }
    }

    pub fn interval_to_units(&self, interval: u32) -> Option<u32> {
        for i in 1..=2 {
            if rc5_multiplier(self.samplerate, i).contains(&interval) {
                return Some(i);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InternalState {
    Idle,
    Data(u8),
    Done,
    Error(Rc5Error),
}

const RISING: bool = true;
const FALLING: bool = false;

impl Receiver for Rc5Receiver {
    type Command = Rc5Command;
    type ReceiveError = Rc5Error;

    fn event(&mut self, pinval: bool, timestamp: u32) -> ReceiverState<Self::Command, Self::ReceiveError> {

        if self.pinval != pinval {
            use InternalState::*;

            let interval = timestamp.wrapping_sub(self.last);
            self.last = timestamp;
            self.pinval = pinval;

            // Number of rc5 units since last pin edge
            let n_units = self.interval_to_units(interval);

            // For debug use
            self.last_interval = interval;
            self.last_state = self.state;

            if let Some(units) = n_units {
                self.rc5_counter += units;
            }

            let odd = self.rc5_counter & 1 == 0;

            let next = match (self.state, pinval, n_units) {
                (Idle, FALLING, _) => Idle,
                (Idle, RISING, _) => {
                    self.data |= 1 << 13;
                    Data(12)
                },
                (Data(0), _, Some(_)) if odd => {
                    self.data |= if pinval {1} else {0};
                    Done
                },
                (Data(n), _, Some(_)) if odd => {
                    self.data |= if pinval {1} else {0} << n;
                    Data(n-1)
                },
                (Data(n), _, Some(_)) => Data(n),
                (Data(_), _, None)    => Error(Rc5Error::Data(interval)),
                (Done, _, _)        => Done,
                (Error(err), _, _)  => Error(err),
            };

            self.state = next;
        }

        match self.state {
            InternalState::Idle => ReceiverState::Idle,
            InternalState::Done => {
                let cmd = Rc5Command::new(self.data);
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
        self.rc5_counter = 0;
    }

    fn disable(&mut self) {
        unimplemented!()
    }
}

const fn rc5_multiplier(samplerate: u32, multiplier: u32) -> Range<u32> {
    let base = (samplerate * 889 * multiplier) / 1000_000;
    range(base, 10)
}

const fn range(len: u32, percent: u32) -> Range<u32> {
    let tol = (len * percent) / 100;

     Range {
        start: len - tol - 2,
        end: len + tol + 2,
    }
}

