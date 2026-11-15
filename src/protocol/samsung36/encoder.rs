use super::{DATA_MARK, HEADER_MARK, HEADER_SPACE, ONE_SPACE, PAUSE_SPACE, ZERO_SPACE};
use crate::{
    protocol::Samsung36,
    sender::{ProtocolEncoder, PulseWriter},
};

impl ProtocolEncoder for Samsung36 {
    // Header, 16 address bits, pause, 20 command bits and the stop mark
    const MAX_PULSES: usize = 2 + 32 + 2 + 40 + 1;

    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize {
        let mut w = PulseWriter::new(buf);

        w.mark(HEADER_MARK);
        w.space(HEADER_SPACE);
        w.pulse_distance(cmd.address.into(), 16, DATA_MARK, ZERO_SPACE, ONE_SPACE);
        w.mark(DATA_MARK);
        w.space(PAUSE_SPACE);
        w.pulse_distance(cmd.command_bits().into(), 20, DATA_MARK, ZERO_SPACE, ONE_SPACE);
        w.mark(DATA_MARK);

        w.finish()
    }
}
