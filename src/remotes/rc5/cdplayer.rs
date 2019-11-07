use crate::{rc5::Rc5Command, ProtocolId};

use crate::remotecontrol_standardbutton;

use crate::remotes::{
    remotecontrol::{DeviceType, StandardButton},
    RemoteControl,
};

remotecontrol_standardbutton!(
    Rc5CdPlayer,
    ProtocolId::Rc5,
    "Marantz CD-Player",
    DeviceType::CDPlayer,
    20,
    Rc5Command,
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
        (37, PitchReset),
        (38, PitchPlus),
        (39, PitchMinus),
        (41, Prog),
        (48, Paus),
        (53, Play),
        (54, Stop),
    ]
);
