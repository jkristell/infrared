use crate::protocols::Rc5;
use crate::send::{InfraredSender, InfraredSenderState};

pub struct Rc5SenderState {
    pub rc5len: u32,
}

impl InfraredSenderState for Rc5SenderState {
    fn create(samplerate: u32) -> Self {
        let rc5len = (889 * samplerate) / 1_000_000;

        Rc5SenderState { rc5len }
    }
}

impl InfraredSender for Rc5 {
    type State = Rc5SenderState;

    fn cmd_pulsedata(state: &Self::State, cmd: &Self::Cmd, buf: &mut [u16]) -> usize {
        // Command as bits
        let bits = cmd.pack();

        let rc5len = state.rc5len as u16;

        // First bit is always one
        buf[0] = 0;
        let mut prev = true;
        let mut index = 1;

        for b in 0..13 {
            let cur = bits & (1 << (12 - b)) != 0;

            if prev == cur {
                buf[index] = rc5len;
                buf[index + 1] = rc5len;
                index += 2;
            } else {
                buf[index] = rc5len * 2;
                index += 1;
            }

            prev = cur;
        }

        index
    }
}
