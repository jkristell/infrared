use crate::RemoteControl;
use crate::remotecontrol::{DeviceType, StandardButton};
use crate::protocols::rc5::Rc5Command;

use crate::standard_mapping;

#[derive(Debug)]
pub struct CdPlayer;

impl RemoteControl<'_, Rc5Command> for CdPlayer {
    type Button = StandardButton;
    const ADDR: u16 = 20;
    const DEVICE: DeviceType = DeviceType::CDPlayer;
    const MODEL: &'static str = "Rc5 CD-player";

    fn decode(&self, raw: Rc5Command) -> Option<Self::Button> {
        if raw.addr as u16 == Self::ADDR {
            to_button(raw.cmd)
        } else {
            None
        }
    }

    fn decode_cmdid(&self, cmdid: u8) -> Option<StandardButton> {
        to_button(cmdid)
    }


    fn encode(&self, button: Self::Button) -> Option<Rc5Command> {
        let addr = Self::ADDR as u8;
        from_button(button).map(|cmd|Rc5Command::new(addr, cmd, false))
    }
}

standard_mapping!(
    [
        (1, One),
        (2, Two),
        (3, Three),
        (4, Four),
        (5, Five),
        (6, Six),
        (7, Seven),
        (8, Eight),
        (9, Nine),

        (11, Time),
        (12, Power),

        (16, Up),
        (17, Down),
        (18, Setup),

        (21, Left),
        (22, Right),
        (23, Enter),
        (28, Random),
        (29, Repeat),

        (32, Next),
        (33, Prev),

        //(37, PitchReset),
        //(38, PitchPlus),
        //(39, PitchMinus),
        //(41, Prog), //

        (48, Paus),

        (53, Play),
        (54, Stop),


    ]
);

