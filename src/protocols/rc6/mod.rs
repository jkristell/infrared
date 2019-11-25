use core::ops::Range;

use crate::prelude::*;
#[cfg(feature = "protocol-dev")]
use crate::ReceiverDebug;
use crate::receiver::ReceiverError;

#[derive(Debug)]
pub struct Rc6Command {
    pub addr: u8,
    pub cmd: u8,
    pub toggle: bool,
}

impl Rc6Command {

    pub fn new(addr: u8, cmd: u8) -> Self {
        Self {
            addr, cmd, toggle: false
        }
    }

    pub fn from_bits(bits: u32, toggle: bool) -> Self {
        let addr = (bits >> 8) as u8;
        let cmd = (bits & 0xFF) as u8;
        Self {addr, cmd, toggle}
    }
}

impl Command for Rc6Command {
    fn construct(addr: u16, cmd: u8) -> Self {
        Rc6Command::new(addr as u8, cmd)
    }

    fn address(&self) -> u16 {
        self.addr as u16
    }

    fn command(&self) -> u8 {
        self.cmd
    }
}


pub struct Rc6 {
    samplerate: u32,
    state: Rc6State,
    pub pinval: bool,
    data: u32,
    headerdata: u32,
    toggle: bool,
    pub last: u32,
    pub rc6_counter: u32,

    #[cfg(feature = "protocol-dev")]
    pub debug: ReceiverDebug<Rc6State, Option<u32>>,
}

impl Rc6 {
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
            #[cfg(feature = "protocol-dev")]
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

        for i in 1..=6 {
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
            Rc6State::Done => Done(Rc6Command::from_bits(self.data, self.toggle)),
            Rc6State::Error(err) => Error(err), //TODO:
            _ => Receiving
        }
    }
}

type Rc6Result = ReceiverState<Rc6Command>;

#[derive(Clone, Copy, Debug)]
pub enum Rc6State {
    Idle,
    Leading,
    LeadingPaus,
    HeaderData(u32),
    Trailing,
    Data(u32),
    Done,
    Error(ReceiverError),
}

const RISING: bool = true;
const FALLING: bool = false;

impl ReceiverStateMachine for Rc6 {
    type Cmd = Rc6Command;
    const ID: ProtocolId = ProtocolId::Rc6;

    fn event(&mut self, rising: bool, sampletime: u32) -> Rc6Result {
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

            (Trailing, FALLING, Some(3))    => { self.toggle = false; Data(15) },
            (Trailing, RISING,  Some(2))    => { self.toggle = true; Data(15) },
            (Trailing, FALLING, Some(1))    => Trailing,
            (Trailing, _, _)                => Idle,

            (Data(0), RISING,   Some(_)) if odd    => Done,
            (Data(0), FALLING,  Some(_)) if odd    => { self.data |= 1; Done },
            (Data(0), _,        Some(_))           => Data(0),
            (Data(n), RISING,   Some(_)) if odd    => Data(n-1),
            (Data(n), FALLING,  Some(_)) if odd    => { self.data |= 1 << n; Data(n-1) },
            (Data(n), _,        Some(_))           => Data(n),
            (Data(_), _,        None)              => Error(ReceiverError::Data(delta as u32)),

            (Done, _, _)        => Done,
            (Error(err), _, _)  => Error(err),
        };

        #[cfg(feature = "protocol-dev")]
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

    fn for_samplerate(samplerate: u32) -> Self {
        Self::new(samplerate)
    }
}

const fn rc6_multiplier(samplerate: u32, multiplier: u32) -> Range<u32> {
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

#[cfg(test)]
mod tests {
    use crate::rc6::Rc6;
    use crate::prelude::*;

    #[test]
    fn basic() {
        let dists = [0, 108,
                     34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19,
                     16, 37, 17, 19, 34, 19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19,
                     16, 37, 34, 20];

        let mut recv = Rc6::new(40_000);
        let mut edge = false;
        let mut tot = 0;
        let mut state = ReceiverState::Idle;

        for dist in dists.iter() {
            edge = !edge;
            tot += dist;
            state = recv.event(edge, tot);
        }

        if let ReceiverState::Done(cmd) = state {
            assert_eq!(cmd.addr, 70);
            assert_eq!(cmd.cmd, 2);
        } else {
            assert!(false);
        }
    }
}



