use crate::remote::RemoteControl;

const SAMSUNGTV_ADDR: u16 = 7;

/// Samsung Tv Remote Control
#[derive(Copy, Clone)]
pub struct SamsungTv;

impl RemoteControl for SamsungTv {
    type Action = SamsungTvAction;

    fn decode(&self, raw: u32) -> Option<SamsungTvAction> {
        let addr = (raw & 0xff) as u16;
        let cmd = ((raw >> 16) & 0xff) as u8;

        if addr != SAMSUNGTV_ADDR {
            return None;
        }

        to_action(cmd)
    }

    fn encode(&self, action: SamsungTvAction) -> u32 {
        let cmd = from_action(action);

        let addr = (SAMSUNGTV_ADDR as u32) | (SAMSUNGTV_ADDR as u32) << 8;
        let cmd = (cmd as u32) << 16 | (!cmd as u32) << 24;

        addr | cmd
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

macro_rules! generate_action_funcs {
    ($out:ty, $t:path, $( ($x:expr, $y:expr, $z:pat) ),* ) => {

        fn to_action(val: u8) -> Option<$out> {
            use $t::*;
            match val {
                $($x => Some($y),)+
                _ => None,
            }
        }

        fn from_action(action: $out) -> u8 {
            use $t::*;
            match action {
                $($z => $x,)+
            }
        }
    };
}

generate_action_funcs!(
    SamsungTvAction,
    SamsungTvAction,
    (2, Power, Power),
    (1, Source, Source),
    (4, One, One),
    (5, Two, Two),
    (6, Three, Three),
    (8, Four, Four),
    (9, Five, Five),
    (10, Six, Six),
    (12, Seven, Seven),
    (13, Eight, Eight),
    (14, Nine, Nine),
    (17, Zero, Zero),
    (44, Teletext, Teletext),
    (19, ChannelPrev, ChannelPrev),
    (7, VolumeUp, VolumeUp),
    (11, VolumeDown, VolumeDown),
    (15, VolumeMute, VolumeMute),
    (107, ChannelList, ChannelList),
    (18, ChannelListNext, ChannelListNext),
    (16, ChannelListPrev, ChannelListPrev),
    (75, Tools, Tools),
    (31, Info, Info),
    (88, Return, Return),
    (45, Exit, Exit),
    (104, Enter, Enter),
    (96, Up, Up),
    (97, Down, Down),
    (101, Left, Left),
    (98, Right, Right),
    (108, Red, Red),
    (20, Green, Green),
    (21, Yellow, Yellow),
    (22, Blue, Blue),
    (63, Emanual, Emanual),
    (62, PictureSize, PictureSize),
    (37, Subtitle, Subtitle),
    (70, Stop, Stop),
    (69, Rewind, Rewind),
    (71, Play, Play),
    (74, Paus, Paus),
    (72, Forward, Forward)
);
