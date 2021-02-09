use crate::protocols::nec::{NecCommandTrait, NecPulseDistance};
use crate::protocols::Nec;
use crate::send::{InfraredSender, InfraredSenderState};

pub struct NecSenderState<Cmd: NecCommandTrait> {
    dists: NecPulseDistance,
    cmd: core::marker::PhantomData<Cmd>,
}

impl<Cmd: NecCommandTrait> InfraredSenderState for NecSenderState<Cmd> {
    fn create(samplerate: u32) -> Self {
        let dists = NecPulseDistance {
            header_high: (samplerate * Cmd::PULSE_DISTANCE.header_high) / 1_000_000,
            header_low: (samplerate * Cmd::PULSE_DISTANCE.header_low) / 1_000_000,
            repeat_low: (samplerate * Cmd::PULSE_DISTANCE.repeat_low) / 1_000_000,
            data_high: (samplerate * Cmd::PULSE_DISTANCE.data_high) / 1_000_000,
            data_zero_low: (samplerate * Cmd::PULSE_DISTANCE.data_zero_low) / 1_000_000,
            data_one_low: (samplerate * Cmd::PULSE_DISTANCE.data_one_low) / 1_000_000,
        };

        NecSenderState {
            dists,
            cmd: Default::default(),
        }
    }
}

impl<Cmd> InfraredSender for Nec<Cmd>
where
    Cmd: NecCommandTrait,
{
    type State = NecSenderState<Cmd>;

    fn cmd_pulsedata(state: &Self::State, cmd: &Self::Cmd, b: &mut [u16]) -> usize {
        b[0] = 0;
        b[1] = state.dists.header_high as u16;
        b[2] = state.dists.header_low as u16;

        let bits = cmd.pack();

        let mut bi = 3;

        for i in 0..32 {
            let one = (bits >> i) & 1 != 0;
            b[bi] = state.dists.data_high as u16;
            if one {
                b[bi + 1] = state.dists.data_one_low as u16;
            } else {
                b[bi + 1] = state.dists.data_zero_low as u16;
            }
            bi += 2;
        }

        bi
    }
}
