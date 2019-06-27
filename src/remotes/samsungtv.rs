use core::convert::From;

use crate::remotes::Remote;

impl Remote for SamsungTv {
    type Action = SamsungTvAction;

    fn action(&self) -> Option<Self::Action> {
        use SamsungTvAction::*;
        let (_addr, cmd) = self.data();

        match cmd {
            2 => Some(Power),
            1 => Some(Source),
            4 => Some(One),
            5 => Some(Two),
            6 => Some(Three),
            8 => Some(Four),
            9 => Some(Five),
            10 => Some(Six),
            12 => Some(Seven),
            13 => Some(Eight),
            14 => Some(Nine),
            17 => Some(Zero),
            44 => Some(Teletext),
            19 => Some(ChannelPrev),
            7 => Some(VolumeUp),
            11 => Some(VolumeDown),
            15 => Some(VolumeMute),
            107 => Some(ChannelList),
            18 => Some(ChannelListNext),
            16 => Some(ChannelListPrev),
            75 => Some(Tools),
            31 => Some(Info),
            88 => Some(Return),
            45 => Some(Exit),
            104 => Some(Enter),
            96 => Some(Up),
            97 => Some(Down),
            101 => Some(Left),
            98 => Some(Right),
            108 => Some(Red),
            20 => Some(Green),
            21 => Some(Yellow),
            22 => Some(Blue),
            63 => Some(Emanual),
            62 => Some(PictureSize),
            37 => Some(Subtitle),
            70 => Some(Stop),
            69 => Some(Rewind),
            71 => Some(Play),
            74 => Some(Paus),
            72 => Some(Forward),
            _ => None,
        }
    }

    fn data(&self) -> (u16, u16) {
        (self.address, self.command)
    }
}

#[derive(Clone, Debug)]
pub struct SamsungTv {
    pub address: u16,
    pub command: u16,
}


impl From<u32> for SamsungTv {
    fn from(value: u32) -> Self {
        Self {
            address: (value & 0xff) as u16,
            command: (value >> 16) as u16,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SamsungTvAction {
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
    VolumeUp,
    VolumeDown,
    VolumeMute,
    ChannelList,
    ChannelListNext,
    ChannelListPrev,
    ChannelPrev,
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
    Forward,
}
