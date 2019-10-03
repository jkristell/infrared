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
    ]
);

