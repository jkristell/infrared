use crate::send::InfraredSender;
use crate::protocols::Nec;
use crate::protocols::nec::{NecCommandTrait, NecTiming};

impl<Cmd, Timing> InfraredSender for Nec<Cmd, Timing>
where
    Cmd: NecCommandTrait<Timing>,
    Timing: NecTiming,
{
    type Cmd = Cmd;

    fn with_samplerate(samplerate: u32) -> Self {
    }

    fn cmd_pulsedata(&self, cmd: &Self::Cmd, b: &mut [u16]) -> usize {
        b[0] = 0;
        b[1] = Timing::PL.hh as u16;
        b[2] = Timing::PL.hl as u16;

        let bits = cmd.pack();

        let mut bi = 3;

        for i in 0..32 {
            let one = (bits >> i) & 1 != 0;
            b[bi] = Timing::PL.dh as u16;
            if one {
                b[bi + 1] = Timing::PL.ol as u16;
            } else {
                b[bi + 1] = Timing::PL.zl as u16;
            }
            bi += 2;
        }

        bi
    }
}