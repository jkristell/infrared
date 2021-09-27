use crate::{
    protocol::{nec::NecCommandVariant, Nec},
    sender::ProtocolEncoder,
};

const fn calc_ticks(l: u32, mut f: u32) -> u32 {
    //TODO: Fix overflow
    let mut div = 1_000_000;

    if f > 1000 {
        f /= 1000;
        div /= 1000;
    }

    f * l / div
}

impl<Cmd, const F: u32> ProtocolEncoder<F> for Nec<Cmd>
where
    Cmd: NecCommandVariant,
{
    type EncoderData = [u32; 6];
    const DATA: [u32; 6] = [
        calc_ticks(Cmd::PULSE_DISTANCE.header_high, F),
        calc_ticks(Cmd::PULSE_DISTANCE.header_low, F),
        calc_ticks(Cmd::PULSE_DISTANCE.repeat_low, F),
        calc_ticks(Cmd::PULSE_DISTANCE.data_high, F),
        calc_ticks(Cmd::PULSE_DISTANCE.data_zero_low, F),
        calc_ticks(Cmd::PULSE_DISTANCE.data_one_low, F),
    ];

    fn encode(cmd: &Self::Cmd, b: &mut [u32]) -> usize {
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
