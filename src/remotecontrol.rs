use crate::protocols::nec::remotes::{SamsungTv, SpecialForMp3};
use crate::protocols::rc5::remotes::CdPlayer;

#[derive(Debug)]
pub enum DeviceType {
    Generic,
    TV,
    DVDPlayer,
    CDPlayer,
    BluRayPlayer,
}

/// A trait describing a Remote Control
pub trait RemoteControl<'a, CMD> {
    /// The type of the buttons
    type Button;

    /// Device adress
    const ADDR: u16;

    /// Type of device that this remote controls
    const DEVICE: DeviceType = DeviceType::Generic;

    /// Remote control model
    const MODEL: &'a str = "<NONAME>";

    /// Try to decode a command into an Button for this remote
    fn decode(&self, raw: CMD) -> Option<Self::Button>;

    fn decode_cmdid(&self, cmdid: u8) -> Option<Self::Button> { None }

    /// Encode a button into a command
    fn encode(&self, button: Self::Button) -> Option<CMD>;
}

#[derive(Debug)]
pub enum Remotes {
    SamsungTv(SamsungTv),
    SpecialForMp3(SpecialForMp3),
    Rc5CdPlayer(CdPlayer),
}

pub const REMOTES: &[Remotes] = &[
    Remotes::SamsungTv(SamsungTv),
    Remotes::SpecialForMp3(SpecialForMp3),
    Remotes::Rc5CdPlayer(CdPlayer),
];

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
/// Extensive list of all buttons ever found on a remote control
pub enum StandardButton {
    Power,
    Source,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,

    Teletext,
    ChannelPrev,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    ChannelList,
    ChannelListNext,
    ChannelListPrev,

    Tools,
    Info,
    Return,
    Exit,
    Enter,
    Up,
    Down,
    Left,
    Right,
    Red,
    Green,
    Yellow,
    Blue,
    Emanual,
    PictureSize,
    Subtitle,
    Stop,
    Rewind,
    Play,
    Paus,
    Play_Paus,
    Forward,
    Mode,
    Shuffle,
    U_SD,
    Plus,
    Minus,
    Next,
    Prev,
    Eq,
    Mute,

}

