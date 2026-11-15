use crate::{
    protocol::Rc6,
    sender::{ProtocolEncoder, PulseWriter},
};

/// Bit period unit in µs
const UNIT: u32 = 444;

impl ProtocolEncoder for Rc6 {
    // Leader, start bit, 3 mode bits, toggle bit and 16 data bits
    const MAX_PULSES: usize = 2 + 2 + 6 + 2 + 32;

    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize {
        let mut w = PulseWriter::new(buf);

        // Leader
        w.mark(6 * UNIT);
        w.space(2 * UNIT);
        // Start bit
        bit(&mut w, true, UNIT);
        // Mode 0
        for _ in 0..3 {
            bit(&mut w, false, UNIT);
        }
        // Toggle bit, double length
        bit(&mut w, cmd.toggle, 2 * UNIT);

        let bits = (u16::from(cmd.addr) << 8) | u16::from(cmd.cmd);
        for b in (0..16).rev() {
            bit(&mut w, bits & (1 << b) != 0, UNIT);
        }

        w.finish()
    }
}

/// Manchester coded bit, a one is a mark followed by a space
fn bit(w: &mut PulseWriter, one: bool, len: u32) {
    w.level(one, len);
    w.level(!one, len);
}
