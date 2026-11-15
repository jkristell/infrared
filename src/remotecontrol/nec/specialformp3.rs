use Action::*;

use crate::{
    protocol::nec::NecCommand,
    remotecontrol::{Action, DeviceType, RemoteControl},
};

/// Generic Mp3 used by me for testing
#[derive(Debug, Default)]
pub struct SpecialForMp3;

impl RemoteControl for SpecialForMp3 {
    const MODEL: &'static str = "Special for Mp3";
    const DEVICE_TYPE: DeviceType = DeviceType::Generic;
    const ADDRESS: u32 = 0;
    type Cmd = NecCommand;

    //type Cmd = NecCommand;
    const BUTTONS: &'static [(u32, Action)] = &[
        (69, Power),
        (70, Mode),
        (71, Mute),
        (68, PlayPause),
        (64, Prev),
        (67, Next),
        (7, Equalizer),
        (21, Minus),
        (9, Plus),
        (22, Zero),
        (25, Shuffle),
        (13, UsbSd),
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
