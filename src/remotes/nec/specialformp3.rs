use crate::{
    remotecontrol::{Button, DeviceType, RemoteControl},
};
use Button::*;
use crate::protocols::nec::cmds::{NecCommand};

/// Generic Mp3 used by me for testing
pub struct SpecialForMp3;

impl RemoteControl for SpecialForMp3 {
    const MODEL: &'static str = "Special for Mp3";
    const DEVTYPE: DeviceType = DeviceType::Generic;
    const ADDRESS: u32 = 0;
    type Cmd = NecCommand;

    //type Cmd = NecCommand;
    const BUTTONS: &'static [(u32, Button)] = &[
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
    ];
}

/*
impl From<NecRaw> for SpecialForMp3Button {
    fn from(cmd: NecRaw) -> Self {
        SpecialForMp3Button(SpecialForMp3::decode(NecStandardCmd::from(cmd)).unwrap())
    }
}

 */