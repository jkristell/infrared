use crate::remote::RemoteControl;

const SAMSUNGTV_ADDR: u16 = 7;

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

    fn encode(&self, cmd: SamsungTvAction) -> u32 {
        let cmd = to_command(cmd);

        let addr = SAMSUNGTV_ADDR as u32;
        let cmd = (cmd as u32) << 16 | (!cmd as u32) << 24;

        addr | cmd
    }
}

fn to_command(action: SamsungTvAction) -> u8 {
    use SamsungTvAction::*;
    match action {
        Power => 2,
        ChannelListNext => 18,
        ChannelListPrev => 16,
        _ => 0,
    }
}


fn to_action(cmd: u8) -> Option<SamsungTvAction> {
    use SamsungTvAction::*;

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


#[derive(Clone, Debug)]
pub struct SamsungTv;


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
