use crate::{protocol::Rc6, sender::ProtocolEncoder};

impl<const F: usize> ProtocolEncoder<F> for Rc6 {
    type EncoderData = [usize; 1];
    const DATA: Self::EncoderData = [((444 * F) / 1_000_000)];

    fn encode(cmd: &Self::Cmd, b: &mut [usize]) -> usize {
        use Level::*;

        let rc6len = <Self as ProtocolEncoder<F>>::DATA[0];

        let header = leader(cmd.toggle, rc6len);

        let bits = u16::from(cmd.addr) << 8 | u16::from(cmd.cmd);
        let payload = payload(bits, rc6len);

        let mut prevlev = Low(0);
        let mut index = 0;

        let low = [Low(6 * (rc6len))];

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
    High(usize),
    Low(usize),
}

/// Construct the leader
const fn leader(toggle: bool, rc6len: usize) -> [Level; 12] {
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

const fn payload(bits: u16, rc6len: usize) -> [Level; 32] {
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
