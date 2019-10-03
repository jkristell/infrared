use crate::remotecontrol::{RemoteControl, StandardButton};
use crate::nec::NecCommand;
use crate::standard_mapping;

#[derive(Debug)]
pub struct SpecialForMp3;

impl RemoteControl<'_, NecCommand> for SpecialForMp3 {
    type Button = StandardButton;
    const ADDR: u16 = 0;
    const MODEL: &'static str = "SpecialForMp3 Remote";

    fn decode(&self, raw: NecCommand) -> Option<StandardButton> {
        if raw.addr as u16 != Self::ADDR {
            return None;
        }
        to_button(raw.cmd)
    }

    fn decode_cmdid(&self, cmdid: u8) -> Option<StandardButton> {
        to_button(cmdid)
    }

    fn encode(&self, button: StandardButton) -> Option<NecCommand> {
        let addr = Self::ADDR as u8;
        from_button(button).map(|cmd| NecCommand { addr, cmd, })
    }
}

standard_mapping!(
    [
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
