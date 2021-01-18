use crate::{
    remotecontrol::{Button, DeviceType, RemoteControl},
    Protocol,
};

use Button::*;
use crate::protocols::nec::cmds::{NecSamsungCommand};

pub struct SamsungTv;

impl RemoteControl for SamsungTv {
    const MODEL: &'static str = "Samsung TV";
    const DEVTYPE: DeviceType = DeviceType::TV;
    const PROTOCOL: Protocol = Protocol::NecSamsung;
    const ADDRESS: u32 = 7;
    type Cmd = NecSamsungCommand;

    const BUTTONS: &'static [(u32, Button)] = &[
        (2, Power),
        (1, Source),
        (4, One),
        (5, Two),
        (6, Three),
        (8, Four),
        (9, Five),
        (10, Six),
        (12, Seven),
        (13, Eight),
        (14, Nine),
        (17, Zero),
        (44, Teletext),
        (19, ChannelPrev),
        (7, VolumeUp),
        (11, VolumeDown),
        (15, VolumeMute),
        (107, ChannelList),
        (18, ChannelListNext),
        (16, ChannelListPrev),
        (75, Tools),
        (31, Info),
        (88, Return),
        (45, Exit),
        (104, Enter),
        (96, Up),
        (97, Down),
        (101, Left),
        (98, Right),
        (108, Red),
        (20, Green),
        (21, Yellow),
        (22, Blue),
        (63, Emanual),
        (62, PictureSize),
        (37, Subtitle),
        (70, Stop),
        (69, Rewind),
        (71, Play),
        (74, Paus),
        (72, Forward),
    ];
}

