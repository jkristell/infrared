use crate::send::InfraredSender;
use crate::protocols::Nec;
use crate::protocols::nec::{NecCommandTrait, NecTiming};

impl<Cmd> InfraredSender for Nec<Cmd>
where
    Cmd: NecCommandTrait + NecTiming,
{
    type Cmd = Cmd;

    fn with_samplerate(samplerate: u32) -> Self {
    }

    fn cmd_pulsedata(&self, cmd: &Self::Cmd, b: &mut [u16]) -> usize {
        b[0] = 0;
        b[1] = Cmd::PD.hh as u16;
        b[2] = Cmd::PD.hl as u16;

        let bits = cmd.pack();

        let mut bi = 3;

        for i in 0..32 {
            let one = (bits >> i) & 1 != 0;
            b[bi] = Cmd::PD.dh as u16;
            if one {
                b[bi + 1] = Cmd::PD.ol as u16;
            } else {
                b[bi + 1] = Cmd::PD.zl as u16;
            }
            bi += 2;
        }

        bi
    }
}