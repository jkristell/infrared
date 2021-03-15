use crate::{
    protocol::{nec::NecCommandVariant, Nec},
    sender::ProtocolEncoder,
};

const fn calc_ticks(l: usize, f: usize) -> usize {
    //TODO: Fix overflow
    f * l / 1_000_000
}

impl<Cmd, const F: usize> ProtocolEncoder<F> for Nec<Cmd>
where
    Cmd: NecCommandVariant,
{
    type EncoderData = [usize; 6];
    const DATA: [usize; 6] = [
        calc_ticks(Cmd::PULSE_DISTANCE.header_high, F),
        calc_ticks(Cmd::PULSE_DISTANCE.header_low, F),
        calc_ticks(Cmd::PULSE_DISTANCE.repeat_low, F),
        calc_ticks(Cmd::PULSE_DISTANCE.data_high, F),
        calc_ticks(Cmd::PULSE_DISTANCE.data_zero_low, F),
        calc_ticks(Cmd::PULSE_DISTANCE.data_one_low, F),
    ];

    fn encode(cmd: &Self::Cmd, b: &mut [usize]) -> usize {
        b[0] = 0;
        b[1] = <Self as ProtocolEncoder<F>>::DATA[0];
        b[2] = <Self as ProtocolEncoder<F>>::DATA[1];

        let bits = cmd.pack();

        let mut bi = 3;

        for i in 0..32 {
            let one = (bits >> i) & 1 != 0;
            b[bi] = <Self as ProtocolEncoder<F>>::DATA[3];
            if one {
                b[bi + 1] = <Self as ProtocolEncoder<F>>::DATA[5];
            } else {
                b[bi + 1] = <Self as ProtocolEncoder<F>>::DATA[4];
            }
            bi += 2;
        }

        bi
    }
}
