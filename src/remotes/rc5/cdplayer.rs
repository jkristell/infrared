use crate::protocols::rc5::Rc5Command;
use crate::remotecontrol::{Button, DeviceType, RemoteControl};
use crate::ProtocolId;
use Button::*;

pub struct CdPlayer;

impl RemoteControl for CdPlayer {
    const MODEL: &'static str = "CD Player (Rc5)";
    const DEVTYPE: DeviceType = DeviceType::CDPlayer;
    const PROTOCOL: ProtocolId = ProtocolId::Rc5;
    const ADDRESS: u32 = 20;
    type Cmd = Rc5Command;
    const BUTTONS: &'static [(u32, Button)] = &[
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
        (37, PitchReset),
        (38, PitchPlus),
        (39, PitchMinus),
        (41, Prog),
        (48, Paus),
        (53, Play),
        (54, Stop),
    ];
}
