use crate::{
    protocol::Rc5,
    sender::{ProtocolEncoder, PulseWriter},
};

/// Half bit period in µs
const HALF_BIT: u32 = 889;

impl ProtocolEncoder for Rc5 {
    // 14 Manchester coded bits
    const MAX_PULSES: usize = 28;

    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize {
        let mut w = PulseWriter::new(buf);

        // The first start bit is always one, followed by the 13 packed bits
        let bits = (1 << 13) | cmd.pack();

        for b in (0..14).rev() {
            // A one is a space followed by a mark
            let one = bits & (1 << b) != 0;
            w.level(!one, HALF_BIT);
            w.level(one, HALF_BIT);
        }

        w.finish()
    }
}
