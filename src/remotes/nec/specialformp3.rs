use crate::remotes::remotecontrol::{DeviceType, RemoteControl, StandardButton};

use crate::remotecontrol_standardbutton;

use crate::nec::NecCommand;
use crate::ProtocolId;

remotecontrol_standardbutton!(
    SpecialForMp3,
    ProtocolId::Nec,
    "Special For Mp3",
    DeviceType::Generic,
    0,
    NecCommand,
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
