use crate::{protocol::Rc5, sender::ProtocolEncoder};

//TODO: Check Overflow
const fn calc_freq(mut f: u32) -> u32 {
    let mut div = 1_000_000;

    if f > 1000 {
        f /= 1000;
        div /= 1000;
    }

    (889 * f) / div
}

impl<const FREQ: u32> ProtocolEncoder<FREQ> for Rc5 {
    type EncoderData = [u32; 1];
    const DATA: Self::EncoderData = [calc_freq(FREQ)];

    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize {
        // Command as bits
        let bits = cmd.pack();

        let rc5len = <Self as ProtocolEncoder<FREQ>>::DATA[0];

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
