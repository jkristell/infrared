use crate::remotecontrol::AsRemoteControlButton;
use crate::PulseLengths;
use core::convert::TryInto;

#[derive(Debug, PartialEq)]
pub struct Rc6Command {
    pub addr: u8,
    pub cmd: u8,
    pub toggle: bool,
}

impl Rc6Command {
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

#[derive(Copy, Clone, PartialEq)]
enum Level {
    High(u16),
    Low(u16),
}

/// Construct the leader
const fn leader(toggle: bool) -> [Level; 12] {
    use Level::*;
    [
        // Leader
        High(6 * 444),
        Low(2 * 444),
        // Start bit after leading pause. Always one
        High(444),
        Low(444),
        // Mode 000
        Low(444),
        High(444),
        Low(444),
        High(444),
        Low(444),
        High(444),
        // Toggle bit. Double length
        if toggle { High(889) } else { Low(889) },
        if toggle { Low(889) } else { High(889) },
    ]
}

const fn payload(bits: u16) -> [Level; 32] {
    let mut lvls = [Level::Low(0); 32];
    let mut i = 0;
    let mut b = 0;
    loop {
        let bit_is_set = bits & (1 << (15 - b)) != 0;
        if bit_is_set {
            lvls[i] = Level::High(444);
            lvls[i + 1] = Level::Low(444);
        } else {
            lvls[i] = Level::Low(444);
            lvls[i + 1] = Level::High(444);
        }

        b += 1;
        i += 2;

        if b == 16 {
            break;
        }
    }
    lvls
}

impl PulseLengths for Rc6Command {
    fn encode(&self, b: &mut [u16]) -> usize {
        use Level::*;

        let header = leader(self.toggle);

        let bits = u16::from(self.addr) << 8 | u16::from(self.cmd);
        let payload = payload(bits);

        let mut prevlev = Low(0);
        let mut index = 0;

        let all = header.iter().chain(payload.iter()).chain(&[Low(6 * 444)]);

        for level in all {
            match (prevlev, *level) {
                (Low(lt), Low(nlt)) => {
                    prevlev = Low(lt + nlt);
                }
                (Low(lt), High(ht)) => {
                    b[index] = lt;
                    index += 1;
                    prevlev = High(ht)
                }
                (High(ht), Low(lt)) => {
                    b[index] = ht;
                    index += 1;
                    prevlev = Low(lt);
                }
                (High(ht), High(nht)) => {
                    prevlev = High(ht + nht);
                }
            }
        }

        index
    }
}

impl AsRemoteControlButton for Rc6Command {
    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn make(addr: u32, cmd: u32) -> Option<Self> {
        Some(Rc6Command::new(addr.try_into().ok()?, cmd.try_into().ok()?))
    }
}
