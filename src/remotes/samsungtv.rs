use core::convert::From;

#[derive(Clone, Debug)]
pub struct SamsungTv {
    pub address: u16,
    pub command: u16,
}

impl SamsungTv {
    pub fn address_command(&self) -> (u16, u8) {
        (self.address, (self.command & 0xFF) as u8)
    }

    pub fn button(&self) -> SamsungTvCommand {
        use SamsungTvCommand::*;

        let (_, cmd) = self.address_command();

        match cmd {
            2 => Power,
            1 => Source,
            4 => One,
            5 => Two,
            6 => Three,
            8 => Four,
            9 => Five,
            10 => Six,
            12 => Seven,
            13 => Eight,
            14 => Nine,
            17 => Zero,
            44 => Teletext,
            19 => ChannelPrev,
            7  => VolumeUp,
            11 => VolumeDown,
            15 => VolumeMute,
            107=> ChannelList,
            18 => ChannelListNext,
            16 => ChannelListPrev,
            75 => Tools,
            31 => Info,
            88 => Return,
            45 => Exit,
            104 => Enter,
            96 => Up,
            97 => Down,
            101 => Left,
            98 => Right,
            108 => Red,
            20 => Green,
            21 => Yellow,
            22 => Blue,
            63 => Emanual,
            62 => PictureSize,
            37 => Subtitle,
            70 => Stop,
            69 => Rewind,
            71 => Play,
            74 => Paus,
            72 => Forward,

            _ => UNKNOWN,
        }
    }
}

impl From<u32> for SamsungTv {
    fn from(value: u32) -> Self {
        Self {
            address: (value & 0xff) as u16,
            command: (value >> 16) as u16
        }
    }
}


#[derive(Clone, Debug)]
pub enum SamsungTvCommand {
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

    UNKNOWN
}
