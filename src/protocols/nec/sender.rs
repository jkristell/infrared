use crate::protocols::nec::{NecCommandTrait, NecPulseDistance, NecTiming};
use crate::protocols::Nec;
use crate::send::{InfraredSender, InfraredSenderState};

pub struct NecSenderState<Cmd: NecTiming> {
    dists: NecPulseDistance,
    _cmd: core::marker::PhantomData<Cmd>,
}

impl<Cmd: NecTiming> InfraredSenderState for NecSenderState<Cmd> {
    fn create(samplerate: u32) -> Self {
        let dists = NecPulseDistance {
            hh: (samplerate * Cmd::PD.hh) / 1_000_000,
            hl: (samplerate * Cmd::PD.hl) / 1_000_000,
            rl: (samplerate * Cmd::PD.rl) / 1_000_000,
            dh: (samplerate * Cmd::PD.dh) / 1_000_000,
            zl: (samplerate * Cmd::PD.zl) / 1_000_000,
            ol: (samplerate * Cmd::PD.ol) / 1_000_000,
        };

        NecSenderState {
            dists,
            _cmd: Default::default(),
        }
    }
}

impl<Cmd> InfraredSender for Nec<Cmd>
where
    Cmd: NecCommandTrait + NecTiming,
{
    type State = NecSenderState<Cmd>;

    fn cmd_pulsedata(state: &Self::State, cmd: &Self::Cmd, b: &mut [u16]) -> usize {
        b[0] = 0;
        b[1] = state.dists.hh as u16;
        b[2] = state.dists.hl as u16;

        let bits = cmd.pack();

        let mut bi = 3;

        for i in 0..32 {
            let one = (bits >> i) & 1 != 0;
            b[bi] = state.dists.dh as u16;
            if one {
                b[bi + 1] = state.dists.ol as u16;
            } else {
                b[bi + 1] = state.dists.zl as u16;
            }
            bi += 2;
        }

        bi
    }
}
