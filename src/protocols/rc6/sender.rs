use crate::protocols::Rc6;
use crate::send::{InfraredSender, InfraredSenderState};

pub struct Rc6SenderState {
    pub rc6len: u32,
}

impl InfraredSenderState for Rc6SenderState {
    fn create(samplerate: u32) -> Self {
        let rc6len = (444 * samplerate) / 1_000_000;

        Rc6SenderState { rc6len }
    }
}

impl InfraredSender for Rc6 {
    type State = Rc6SenderState;

    fn cmd_pulsedata(state: &Self::State, cmd: &Self::Cmd, b: &mut [u16]) -> usize {
        use Level::*;

        let header = leader(cmd.toggle, state.rc6len as u16);

        let bits = u16::from(cmd.addr) << 8 | u16::from(cmd.cmd);
        let payload = payload(bits, state.rc6len as u16);

        let mut prevlev = Low(0);
        let mut index = 0;

        let low = [Low(6 * (state.rc6len as u16))];

        let all = header.iter().chain(payload.iter()).chain(&low);

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

#[derive(Copy, Clone, PartialEq)]
enum Level {
    High(u16),
    Low(u16),
}

/// Construct the leader
const fn leader(toggle: bool, rc6len: u16) -> [Level; 12] {
    use Level::*;
    [
        // Leader
        High(6 * rc6len),
        Low(2 * rc6len),
        // Start bit after leading pause. Always one
        High(rc6len),
        Low(rc6len),
        // Mode 000
        Low(rc6len),
        High(rc6len),
        Low(rc6len),
        High(rc6len),
        Low(rc6len),
        High(rc6len),
        // Toggle bit. Double length
        if toggle {
            High(2 * rc6len)
        } else {
            Low(2 * rc6len)
        },
        if toggle {
            Low(2 * rc6len)
        } else {
            High(2 * rc6len)
        },
    ]
}

const fn payload(bits: u16, rc6len: u16) -> [Level; 32] {
    let mut lvls = [Level::Low(0); 32];
    let mut i = 0;
    let mut b = 0;
    loop {
        let bit_is_set = bits & (1 << (15 - b)) != 0;
        if bit_is_set {
            lvls[i] = Level::High(rc6len);
            lvls[i + 1] = Level::Low(rc6len);
        } else {
            lvls[i] = Level::Low(rc6len);
            lvls[i + 1] = Level::High(rc6len);
        }

        b += 1;
        i += 2;

        if b == 16 {
            break;
        }
    }
    lvls
}
