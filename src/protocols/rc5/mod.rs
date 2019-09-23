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

        Self {addr, cmd, start, toggle}
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc5Error {
    Header(u16),
    Data(u16),
    Rc5Version(u16),
}

pub struct Rc5Receiver {
    samplerate: u32,
    state: InternalState,
    pinval: bool,
    data: u16,
    last: u32,
    pub rc5_counter: u32,

    pub last_interval: u16,
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

    pub fn interval_to_units(&self, interval: u16) -> Option<u32> {
        for i in 1..=2 {
            if rc5_multiplier(self.samplerate, i).contains(&(interval as u32)) {
                return Some(i);
            }
        }
        None
    }

    fn internal_state_to_receiver_state(&self) -> Rc5RS {
        match self.state {
            InternalState::Idle => ReceiverState::Idle,
            InternalState::Done => {
                let cmd = Rc5Command::new(self.data);
                ReceiverState::Done(cmd)
            },
            InternalState::Error(err) => ReceiverState::Error(err),
            _ => ReceiverState::Receiving
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InternalState {
    Idle,
    Data(u8),
    Done,
    Error(Rc5Error),
    Disabled,
}

const RISING: bool = true;
const FALLING: bool = false;

type Rc5RS = ReceiverState<Rc5Command, Rc5Error>;

impl Receiver for Rc5Receiver {
    type Cmd = Rc5Command;
    type Err = Rc5Error;

    fn sample(&mut self, pinval: bool, timestamp: u32) -> ReceiverState<Self::Cmd, Self::Err> {

        if self.pinval != pinval {

            let interval = timestamp.wrapping_sub(self.last);
            self.last = timestamp;
            self.pinval = pinval;

            if interval == 0 || interval == timestamp ||  interval >= core::u16::MAX.into() {
                return self.internal_state_to_receiver_state();
            }

            let interval = interval as u16;
            return self.edge(pinval, interval);
        }

        self.internal_state_to_receiver_state()
    }

    fn edge(&mut self, rising: bool, sampledelta: u16) -> ReceiverState<Self::Cmd, Self::Err> {
        use InternalState::*;

        // Number of rc5 units since last pin edge
        let n_units = self.interval_to_units(sampledelta);

        // For debug use
        self.last_interval = sampledelta;
        self.last_state = self.state;

        if let Some(units) = n_units {
            self.rc5_counter += units;
        }

        let odd = self.rc5_counter & 1 == 0;

        self.state = match (self.state, rising, n_units) {
            (Idle, FALLING, _) => Idle,
            (Idle, RISING, _) => {
                self.data |= 1 << 13;
                Data(12)
            },
            (Data(0), _, Some(_)) if odd => {
                self.data |= if rising {1} else {0};
                Done
            },
            (Data(n), _, Some(_)) if odd => {
                self.data |= if rising {1} else {0} << n;
                Data(n-1)
            },
            (Data(n), _, Some(_)) => Data(n),
            (Data(_), _, None)    => Error(Rc5Error::Data(sampledelta)),
            (Done, _, _)        => Done,
            (Error(err), _, _)  => Error(err),
            (Disabled, _, _) => Disabled,
        };

        self.internal_state_to_receiver_state()
    }

    fn reset(&mut self) {
        self.state = InternalState::Idle;
        self.pinval = false;
        self.data = 0;
        self.rc5_counter = 0;
    }

    fn disable(&mut self) {
        self.state = InternalState::Disabled;
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

