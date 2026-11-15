use Action::*;

use crate::{
    protocol::samsung36::Samsung36Command,
    remotecontrol::{Action, DeviceType, RemoteControl},
};

#[derive(Debug, Default)]
pub struct SamsungBluRayPlayer;

impl RemoteControl for SamsungBluRayPlayer {
    const MODEL: &'static str = "Samsung BluRay Player";
    const DEVICE_TYPE: DeviceType = DeviceType::BluRayPlayer;
    const ADDRESS: u32 = 32;
    type Cmd = Samsung36Command;
    const BUTTONS: &'static [(u32, Action)] = &[
        (2, One),
        (3, Two),
        (4, Three),
        (5, Four),
        (6, Five),
        (7, Six),
        (8, Seven),
        (9, Eight),
        (10, Nine),
    ];
}
