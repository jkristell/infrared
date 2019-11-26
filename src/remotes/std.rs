use crate::remotes::{
    nec::{SamsungTv, SpecialForMp3},
    rc5::Rc5CdPlayer,
    sbp::SamsungBluRayPlayer,
    DeviceType, RemoteControl, StandardButton,
};
use crate::ProtocolId;

pub fn remotes() -> Vec<RemoteControlData> {
    // Pretty much every remote ever manufactured :-)
    vec![
        RemoteControlData::construct::<Rc5CdPlayer>(),
        RemoteControlData::construct::<SamsungTv>(),
        RemoteControlData::construct::<SpecialForMp3>(),
        RemoteControlData::construct::<SamsungBluRayPlayer>(),
    ]
}

#[derive(Debug)]
pub struct RemoteControlData {
    pub model: &'static str,
    pub addr: u16,
    pub protocol: ProtocolId,
    pub dtype: DeviceType,
    pub mapping: &'static [(u8, StandardButton)],
}

impl RemoteControlData {
    pub fn construct<REMOTE>() -> RemoteControlData
    where
        REMOTE: RemoteControl,
    {
        RemoteControlData {
            addr: REMOTE::ADDR,
            model: REMOTE::MODEL,
            dtype: REMOTE::DEVICE,
            protocol: REMOTE::PROTOCOL_ID,
            mapping: REMOTE::MAPPING,
        }
    }
}
