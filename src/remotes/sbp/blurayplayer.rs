use crate::{ProtocolId};
use crate::protocols::sbp::SbpCommand;

use crate::remotecontrol_standardbutton;

use crate::remotes::{
    remotecontrol::{DeviceType, StandardButton},
    RemoteControl,
};

remotecontrol_standardbutton!(
    SamsungBluRayPlayer,
    ProtocolId::Sbp,
    "Samsung Bluray Player",
    DeviceType::BluRayPlayer,
    32,
    SbpCommand,
    [
        (2, One),
        (3, Two),
        (4, Three),
        (5, Four),
        (6, Five),
        (7, Six),
        (8, Seven),
        (9, Eight),
        (10, Nine),
    ]
);
