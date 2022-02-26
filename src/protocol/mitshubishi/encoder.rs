use crate::{
    sender::ProtocolEncoder,
};
use crate::protocol::Mitsubishi;

const fn calc_ticks(l: u32, mut f: u32) -> u32 {
    //TODO: Fix overflow
    let mut div = 1_000_000;

    if f > 1000 {
        f /= 1000;
        div /= 1000;
    }

    f * l / div
}


const PULSE_DATA: [u32; 7] = [
    3450,
    1600,
    440,
    1200,   // 0
    400,    // 1
    440,
    11300,
];

fn byte_to_ts<const F: u32>(bits: u8, b: &mut [u32]) {

    let mut bi = 0;

    for i in 0..8 {
        let one = (bits >> i) & 1 != 0;
        b[bi] = <Mitsubishi as ProtocolEncoder<F>>::DATA[2];
        if one {
            b[bi + 1] = <Mitsubishi as ProtocolEncoder<F>>::DATA[3];
        } else {
            b[bi + 1] = <Mitsubishi as ProtocolEncoder<F>>::DATA[4];
        }
        bi += 2;
    }

}

impl<const F: u32> ProtocolEncoder<F> for Mitsubishi
{
    type EncoderData = [u32; 7];
    const DATA: [u32; 7] = [
        calc_ticks(PULSE_DATA[0], F),
        calc_ticks(PULSE_DATA[1], F),
        calc_ticks(PULSE_DATA[2], F),
        calc_ticks(PULSE_DATA[3], F),
        calc_ticks(PULSE_DATA[4], F),
        calc_ticks(PULSE_DATA[5], F),
        calc_ticks(PULSE_DATA[6], F),
    ];

    fn encode(cmd: &Self::Cmd, b: &mut [u32]) -> usize {
        b[0] = 0;
        b[1] = <Self as ProtocolEncoder<F>>::DATA[0];
        b[2] = <Self as ProtocolEncoder<F>>::DATA[1];

        let bytes = cmd.pack();

        let mut bi = 3;

        for bb in bytes {
            byte_to_ts::<F>(bb, &mut b[bi..]);
            bi += 2 * 8;
        }

        b[bi + 0] = <Self as ProtocolEncoder<F>>::DATA[5];
        b[bi + 1] = <Self as ProtocolEncoder<F>>::DATA[6];

        b[bi + 2] = <Self as ProtocolEncoder<F>>::DATA[0];
        b[bi + 3] = <Self as ProtocolEncoder<F>>::DATA[1];
        bi += 4;

        for bb in bytes {
            byte_to_ts::<F>(bb, &mut b[bi..]);
            bi += 2 * 8;
        }

        bi

    }


}

