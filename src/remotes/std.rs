use crate::{
    remotecontrol::{Button, DeviceType, RemoteControl},
    remotes::{
        nec::{SamsungTv, SpecialForMp3},
        rc5::CdPlayer,
        sbp::SamsungBluRayPlayer,
    },
    ProtocolId,
};

pub fn remotes() -> Vec<RemoteControlData> {
    // Pretty much every remote ever manufactured :-)
    vec![
        RemoteControlData::new::<CdPlayer>(),
        RemoteControlData::new::<SamsungTv>(),
        RemoteControlData::new::<SpecialForMp3>(),
        RemoteControlData::new::<SamsungBluRayPlayer>(),
    ]
}

#[derive(Debug)]
pub struct RemoteControlData {
    pub model: &'static str,
    pub addr: u32,
    pub protocol: ProtocolId,
    pub dtype: DeviceType,
    pub mapping: &'static [(u32, Button)],
}

impl RemoteControlData {
    pub fn new<R>() -> RemoteControlData
    where
        R: RemoteControl,
    {
        RemoteControlData {
            addr: R::ADDRESS,
            model: R::MODEL,
            dtype: R::DEVTYPE,
            mapping: R::BUTTONS,
            protocol: R::PROTOCOL,
        }
    }
}
