//! Rc6

use core::convert::TryInto;
use core::ops::Range;

use crate::{
    recv::{Error, ReceiverSM, State},
    Command,
    cmd::Protocol,
};

#[derive(Debug)]
pub struct Rc6Cmd {
    pub addr: u8,
    pub cmd: u8,
    pub toggle: bool,
}

impl Rc6Cmd {
    pub fn new(addr: u8, cmd: u8) -> Self {
        Self {
            addr,
            cmd,
            toggle: false,
        }
    }

    pub fn from_bits(bits: u32, toggle: bool) -> Self {
        let addr = (bits >> 8) as u8;
        let cmd = (bits & 0xFF) as u8;
        Self { addr, cmd, toggle }
    }
}

impl Command for Rc6Cmd {
    fn construct(addr: u32, cmd: u32) -> Option<Self> {
        Some(Rc6Cmd::new(addr.try_into().ok()?, cmd.try_into().ok()?))
    }

    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn data(&self) -> u32 {
        self.cmd.into()
    }

    fn protocol(&self) -> Protocol {
        Protocol::Rc6
    }

}

#[derive(Default)]
pub struct Rc6 {
    state: Rc6State,
    data: u32,
    headerdata: u32,
    toggle: bool,
    rc6_counter: u32,
}

impl Rc6 {
    fn interval_to_units(&self, interval: u16) -> Option<u32> {
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

impl From<Rc6State> for State {
    fn from(state: Rc6State) -> Self {
        use Rc6State::*;
        match state {
            Idle => State::Idle,
            Done => State::Done,
            Rc6Err(err) => State::Error(err),
            Leading | LeadingPaus | HeaderData(_) | Trailing | Data(_) => State::Receiving,
        }
    }
}

const RISING: bool = true;
const FALLING: bool = false;

impl ReceiverSM for Rc6 {
    type Cmd = Rc6Cmd;
    type InternalState = Rc6State;

    fn create() -> Self {
        Self::default()
    }

    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: u32) -> Rc6State {
        use Rc6State::*;

        // Number of rc6 units since last pin edge
        let n_units = self.interval_to_units(dt as u16);

        if let Some(units) = n_units {
            self.rc6_counter += units;
        } else {
            self.reset();
        }

        let odd = self.rc6_counter & 1 == 1;

        self.state = match (self.state, rising, n_units) {
            (Idle,          FALLING,    _)          => Idle,
            (Idle,          RISING,     _)          => { self.rc6_counter = 0; Leading },
            (Leading,       FALLING,    Some(6))    => LeadingPaus,
            (Leading,       _,          _)          => Idle,
            (LeadingPaus,   RISING,     Some(2))    => HeaderData(3),
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

            (Trailing,      FALLING,    Some(3))    => { self.toggle = false; Data(15) }
            (Trailing,      RISING,     Some(2))    => { self.toggle = true; Data(15) }
            (Trailing,      FALLING,    Some(1))    => Trailing,
            (Trailing,      _,          _)          => Idle,

            (Data(0),       RISING,     Some(_)) if odd => Done,
            (Data(0),       FALLING,    Some(_)) if odd => { self.data |= 1; Done }
            (Data(0),       _,          Some(_))    => Data(0),
            (Data(n),       RISING,     Some(_)) if odd => Data(n - 1),
            (Data(n),       FALLING,    Some(_)) if odd => { self.data |= 1 << n; Data(n - 1) }
            (Data(n),       _,          Some(_))    => Data(n),
            (Data(_),       _,          None)       => Rc6Err(Error::Data),

            (Done,          _,          _)          => Done,
            (Rc6Err(err),    _,          _)         => Rc6Err(err),
        };

        self.state
    }

    fn command(&self) -> Option<Self::Cmd> {
        Some(Rc6Cmd::from_bits(self.data, self.toggle))
    }

    fn reset(&mut self) {
        self.state = Rc6State::Idle;
        self.data = 0;
        self.headerdata = 0;
        self.rc6_counter = 0;
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

#[cfg(test)]
mod tests {
    use crate::protocols::rc6::Rc6;
    use crate::recv::*;

    #[test]
    fn basic() {
        let dists = [
            0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19,
            34, 19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,

            0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19,
            34, 19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,

        ];

        let mut recv = EventReceiver::<Rc6>::new(40_000);
        let mut edge = false;
        let mut tot = 0;

        for dist in dists.iter() {
            edge = !edge;
            tot += dist;

            let s0 = recv.sm.state;

            let cmd = recv.edge_event(edge, tot);

            println!(
                "{} ({}): {:?} -> {:?}",
                edge as u32,
                dist,
                s0,
                recv.sm.state
            );


            if let Ok(Some(cmd)) = cmd {
                println!("cmd: {:?}", cmd);
                assert_eq!(cmd.addr, 70);
                assert_eq!(cmd.cmd, 2);
            }
        }
    }
}
