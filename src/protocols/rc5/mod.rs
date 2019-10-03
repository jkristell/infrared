
pub mod receiver;
pub mod transmitter;

pub mod remotes;


pub use receiver::{Rc5Receiver};
pub use transmitter::Rc5Transmitter;

const ADDR_MASK: u16   = 0b_0000_0111_1100_0000;
const CMD_MASK: u16    = 0b_0000_0000_0011_1111;
const START_MASK: u16  = 0b_0011_0000_0000_0000;
const TOGGLE_MASK: u16 = 0b_0000_1000_0000_0000;

const ADDR_SHIFT: u32 = 6;
const START_SHIFT: u32 = 12;
const TOGGLE_SHIFT: u32 = 11;

#[derive(Debug, Eq, PartialEq)]
pub struct Rc5Command {
    pub addr: u8,
    pub cmd: u8,
    pub start: u8,
    pub toggle: u8,
}

impl Rc5Command {
    pub const fn from_bits(data: u16) -> Self {

        let addr = ((data & ADDR_MASK) >> ADDR_SHIFT) as u8;
        let cmd = (data & CMD_MASK) as u8;
        let start = ((data & START_MASK) >> START_SHIFT) as u8;
        let toggle = ((data & TOGGLE_MASK) >> TOGGLE_SHIFT) as u8;

        Self {addr, cmd, start, toggle}
    }

    pub const fn new(addr: u8, cmd: u8, toggle: bool) -> Self {
        Self {
            addr,
            cmd,
            start: 0b11,
            toggle: toggle as u8,
        }
    }

    pub fn to_bits(&self) -> u16 {

        u16::from(self.addr) << ADDR_SHIFT |
        u16::from(self.cmd) |
        u16::from(self.toggle) << TOGGLE_SHIFT |
        u16::from(self.start) << START_SHIFT
    }
}



#[cfg(test)]
mod tests {
    use crate::rc5::Rc5Receiver;
    use crate::prelude::*;
    use crate::protocols::rc5::{Rc5Command, Rc5Transmitter};

    #[test]
    fn rc5_command() {
        let cmd = Rc5Command::new(20, 15, false);
        assert_eq!(cmd, Rc5Command::from_bits(cmd.to_bits()))
    }

    #[test]
    fn command() {

        let dists = [0, 37, 34,
            72, 72, 73, 70, 72,
            36, 37, 34, 36, 36, 36,

            71, 73,
            35, 37,
            70, 37];

        let mut recv = Rc5Receiver::new(40_000);
        let mut edge = false;
        let mut tot = 0;
        let mut state = ReceiverState::Idle;

        for dist in dists.iter() {
            edge = !edge;
            tot += dist;
            state = recv.sample_edge(edge, tot);
        }

        if let ReceiverState::Done(cmd) = state {
            assert_eq!(cmd.addr, 20);
            assert_eq!(cmd.cmd, 9);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn rc5_transmit() {

        let mut tx = Rc5Transmitter::new_for_samplerate(40_000);

        tx.load(Rc5Command::new(20, 9, false));

        println!("bits: {:X?}", tx.bits);

        let mut last_enable = false;
        let mut last_ts = 0;

        for ts in 0..2000 {
            let state = tx.step(ts);

            if let TransmitterState::Transmit(v) = state {
                if v != last_enable {
                    last_enable = v;
                    let delta = ts - last_ts;
                    println!("state: {}: {:?}", delta, state);
                    last_ts = ts;
                }
            }
        }
    }
}

