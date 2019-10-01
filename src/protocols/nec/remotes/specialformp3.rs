use crate::remotecontrol::RemoteControl;
use crate::nec::NecCommand;
use crate::nec_buttons;

const ADDR: u8 = 0;

pub struct SpecialForMp3;

impl RemoteControl<'_, NecCommand> for SpecialForMp3 {
    type Button = SpecialForMp3Button;

    fn decode(&self, raw: NecCommand) -> Option<SpecialForMp3Button> {
        if raw.addr != ADDR {
            return None;
        }
        to_button(raw.cmd)
    }

    fn encode(&self, button: SpecialForMp3Button) -> NecCommand {
        NecCommand {
            addr: ADDR,
            cmd: from_button(button),
        }
    }
}

nec_buttons!(
    SpecialForMp3Button, [
        (69, Power),
        (70, Mode),
        (71, Mute),
        (68, Play_Paus),
        (64, Prev),
        (67, Next),
        (7, Eq),
        (21, Minus),
        (9, Plus),
        (22, Zero),
        (25, Shuffle),
        (13, U_SD),
        (12, One),
        (24, Two),
        (94, Three),
        (8, Four),
        (28, Five),
        (90, Six),
        (66, Seven),
        (82, Eight),
        (74, Nine),
    ]
);
