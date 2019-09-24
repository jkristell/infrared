use crate::remote::RemoteControl;
use crate::protocols::nec::NecCommand;

const SAMSUNGTV_ADDR: u16 = 7;

#[derive(Copy, Clone)]
/// Samsung Tv Remote Control
pub struct SamsungTv;

impl RemoteControl<NecCommand> for SamsungTv {
    type Action = SamsungTvAction;

    fn decode(&self, cmd: NecCommand) -> Option<SamsungTvAction> {
        if cmd.addr != SAMSUNGTV_ADDR {
            return None;
        }

        to_action(cmd.cmd as u8)
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
    ($out:tt, $( ($x:expr, $y:tt) ),* ) => {

        fn to_action(val: u8) -> Option<$out> {
            use $out::*;
            match val {
                $($x => Some($y),)+
                _ => None,
            }
        }

        fn from_action(action: $out) -> u8 {
            use $out::*;
            match action {
                $($y => $x,)+
            }
        }
    };
}

generate_action_funcs!(
    SamsungTvAction,
    (2, Power),
    (1, Source),
    (4, One),
    (5, Two),
    (6, Three),
    (8, Four),
    (9, Five),
    (10, Six),
    (12, Seven),
    (13, Eight),
    (14, Nine),
    (17, Zero),
    (44, Teletext),
    (19, ChannelPrev),
    (7, VolumeUp),
    (11, VolumeDown),
    (15, VolumeMute),
    (107, ChannelList),
    (18, ChannelListNext),
    (16, ChannelListPrev),
    (75, Tools),
    (31, Info),
    (88, Return),
    (45, Exit),
    (104, Enter),
    (96, Up),
    (97, Down),
    (101, Left),
    (98, Right),
    (108, Red),
    (20, Green),
    (21, Yellow),
    (22, Blue),
    (63, Emanual),
    (62, PictureSize),
    (37, Subtitle),
    (70, Stop),
    (69, Rewind),
    (71, Play),
    (74, Paus),
    (72, Forward)
);
