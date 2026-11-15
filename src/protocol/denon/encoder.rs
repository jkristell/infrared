use super::{DATA_HIGH, HEADER_HIGH, HEADER_LOW, ONE_LOW, ZERO_LOW};
use crate::{
    protocol::Denon,
    sender::{ProtocolEncoder, PulseWriter},
};

impl ProtocolEncoder for Denon {
    // Header, 48 bits and the stop mark
    const MAX_PULSES: usize = 2 + 96 + 1;

    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize {
        let mut w = PulseWriter::new(buf);

        w.mark(HEADER_HIGH);
        w.space(HEADER_LOW);
        w.pulse_distance(cmd.pack(), 48, DATA_HIGH, ZERO_LOW, ONE_LOW);
        w.mark(DATA_HIGH);

        w.finish()
    }
}
