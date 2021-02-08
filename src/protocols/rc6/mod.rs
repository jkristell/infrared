//! Philips Rc6

use core::ops::Range;

use crate::recv::{Error, InfraredReceiver, Status};

mod cmd;
pub use cmd::Rc6Command;

#[cfg(test)]
mod tests;

#[derive(Default)]
/// Philips Rc6
pub struct Rc6 {
    state: Rc6State,
    data: u32,
    headerdata: u32,
    toggle: bool,
    clock: u32,
}

impl Rc6 {
    pub fn interval_to_units(interval: u16) -> Option<u32> {
        let interval = u32::from(interval);

        for i in 1..=6 {
            if rc6_multiplier(i).contains(&interval) {
                return Some(i);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc6State {
    Idle,
    Leading,
    LeadingPaus,
    HeaderData(u32),
    Trailing,
    Data(u32),
    Done,
    Rc6Err(Error),
}

impl Default for Rc6State {
    fn default() -> Self {
        Rc6State::Idle
    }
}

impl From<Rc6State> for Status {
    fn from(state: Rc6State) -> Self {
        use Rc6State::*;
        match state {
            Idle => Status::Idle,
            Done => Status::Done,
            Rc6Err(err) => Status::Error(err),
            _ => Status::Receiving,
        }
    }
}

impl InfraredReceiver for Rc6 {
    type Cmd = Rc6Command;
    type InternalState = Rc6State;

    fn create_receiver() -> Self {
        Self::default()
    }

    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: u32) -> Rc6State {
        use Rc6State::*;

        // Number of rc6 clock ticks since last edge
        let ticks = Rc6::interval_to_units(dt as u16);

        // Reconstruct the clock
        if let Some(t) = ticks {
            self.clock += t;
        } else {
            self.reset();
        }

        let odd = self.clock & 1 == 1;

        self.state = match (self.state, rising, ticks) {
            (Idle,          false,    _)            => Idle,
            (Idle,          true,     _)            => { self.clock = 0; Leading },
            (Leading,       false,    Some(6))      => LeadingPaus,
            (Leading,       _,          _)          => Idle,
            (LeadingPaus,   true,     Some(2))      => HeaderData(3),
            (LeadingPaus,   _,          _)          => Idle,

            (HeaderData(n), _,          Some(_)) if odd => {
                self.headerdata |= if rising { 0 } else { 1 } << n;
                if n == 0 {
                    Trailing
                } else {
                    HeaderData(n - 1)
                }
            }

            (HeaderData(n), _,          Some(_))    => HeaderData(n),
            (HeaderData(_), _,          None)       => Idle,

            (Trailing,      false,      Some(3))    => { self.toggle = true; Data(15) }
            (Trailing,      true,       Some(2))    => { self.toggle = false; Data(15) }
            (Trailing,      false,      Some(1))    => Trailing,
            (Trailing,      _,          _)          => Idle,

            (Data(0),       true,       Some(_)) if odd => Done,
            (Data(0),       false,      Some(_)) if odd => { self.data |= 1; Done }
            (Data(0),       _,          Some(_))    => Data(0),
            (Data(n),       true,       Some(_)) if odd => Data(n - 1),
            (Data(n),       false,      Some(_)) if odd => { self.data |= 1 << n; Data(n - 1) }
            (Data(n),       _,          Some(_))    => Data(n),
            (Data(_),       _,          None)       => Rc6Err(Error::Data),

            (Done,          _,          _)          => Done,
            (Rc6Err(err),   _,          _)          => Rc6Err(err),
        };

        self.state
    }

    fn command(&self) -> Option<Self::Cmd> {
        Some(Rc6Command::from_bits(self.data, self.toggle))
    }

    fn reset(&mut self) {
        self.state = Rc6State::Idle;
        self.data = 0;
        self.headerdata = 0;
        self.clock = 0;
    }
}

const fn rc6_multiplier(multiplier: u32) -> Range<u32> {
    let base = 444 * multiplier;
    range(base, 12)
}

const fn range(len: u32, percent: u32) -> Range<u32> {
    let tol = (len * percent) / 100;

    Range {
        start: len - tol - 2,
        end: len + tol + 4,
    }
}
