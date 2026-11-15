use crate::{
    protocol::{Nec, nec::NecCommandVariant},
    sender::{ProtocolEncoder, PulseWriter},
};

impl<Cmd> ProtocolEncoder for Nec<Cmd>
where
    Cmd: NecCommandVariant,
{
    // Header, 32 bits and the stop mark
    const MAX_PULSES: usize = 2 + 64 + 1;

    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize {
        let t = Cmd::PULSE_DISTANCE;
        let mut w = PulseWriter::new(buf);

        w.mark(t.header_high);
        if cmd.repeat() && Cmd::SHORT_REPEAT {
            w.space(t.repeat_low);
        } else {
            w.space(t.header_low);
            w.pulse_distance(
                cmd.pack().into(),
                32,
                t.data_high,
                t.data_zero_low,
                t.data_one_low,
            );
        }
        // Stop mark, terminating the last space
        w.mark(t.data_high);

        w.finish()
    }
}
