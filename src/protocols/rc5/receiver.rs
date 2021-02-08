use core::ops::Range;

use crate::{
    protocols::rc5::Rc5Command,
    recv::{Error, InfraredReceiver, Status},
};

#[derive(Default)]
/// Philips Rc5
pub struct Rc5 {
    pub(crate) state: Rc5State,
    bitbuf: u16,
    pub(crate) clock: u32,
}

impl Rc5 {
    pub fn interval_to_units(&self, interval: u32) -> Option<u32> {
        for i in 1..=2 {
            if rc5_multiplier(i).contains(&interval) {
                return Some(i);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc5State {
    Idle,
    Data(u8),
    Done,
    Err(Error),
}

impl Default for Rc5State {
    fn default() -> Self {
        Rc5State::Idle
    }
}

impl From<Rc5State> for Status {
    fn from(rs: Rc5State) -> Self {
        use Rc5State::*;
        match rs {
            Idle => Status::Idle,
            Data(_) => Status::Receiving,
            Done => Status::Done,
            Err(e) => Status::Error(e),
        }
    }
}

impl InfraredReceiver for Rc5 {
    type Cmd = Rc5Command;
    type InternalState = Rc5State;

    fn create_receiver() -> Self {
        Rc5::default()
    }

    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: u32) -> Self::InternalState {
        use Rc5State::*;

        // Number of rc5 units since last pin edge
        let rc5units = self.interval_to_units(dt);

        if let Some(units) = rc5units {
            self.clock += units;
        } else {
            self.reset();
        }

        let is_odd = self.clock & 1 == 0;

        self.state = match (self.state, rising, rc5units) {

            (Idle,      false,    _) => Idle,
            (Idle,      true,     _) => {
                self.clock = 0;
                self.bitbuf |= 1 << 13; Data(12)
            }

            (Data(0),   true,     Some(_)) if is_odd => { self.bitbuf |= 1; Done }
            (Data(0),   false,    Some(_)) if is_odd => Done,

            (Data(bit), true,     Some(_)) if is_odd => { self.bitbuf |= 1 << bit; Data(bit - 1) }
            (Data(bit), false,    Some(_)) if is_odd => Data(bit - 1),

            (Data(bit), _,          Some(_)) => Data(bit),
            (Data(_),   _,          None) => Err(Error::Data),
            (Done,      _,          _) => Done,
            (Err(err),  _,          _) => Err(err),
        };

        self.state
    }

    fn command(&self) -> Option<Self::Cmd> {
        Some(Rc5Command::unpack(self.bitbuf))
    }

    fn reset(&mut self) {
        self.state = Rc5State::Idle;
        self.bitbuf = 0;
        self.clock = 0;
    }
}

const fn rc5_multiplier(multiplier: u32) -> Range<u32> {
    let base = 889 * multiplier;
    range(base, 10)
}

const fn range(len: u32, percent: u32) -> Range<u32> {
    let tol = (len * percent) / 100;

    Range {
        start: len - tol - 2,
        end: len + tol + 2,
    }
}
