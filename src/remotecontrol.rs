use crate::{Command, Protocol};

#[derive(Debug)]
/// Device type that the remote control controls
pub enum DeviceType {
    Generic,
    TV,
    DVDPlayer,
    CDPlayer,
    BluRayPlayer,
}

/// A trait describing a Remote Control
pub trait RemoteControl {
    /// Remote control model
    const MODEL: &'static str = "<NONAME>";
    /// Type of device that this remote controls
    const DEVTYPE: DeviceType = DeviceType::Generic;
    /// Protocol
    const PROTOCOL: Protocol = Protocol::Unknown;
    /// Device address
    const ADDRESS: u32;
    /// The type of command
    type Cmd: Command;
    /// command byte to standardbutton mapping
    const BUTTONS: &'static [(u8, Button)] = &[];
    /// Try to map a command into an Button for this remote
    fn decode(cmd: Self::Cmd) -> Option<Button> {
        // Check address
        if Self::ADDRESS != cmd.address() {
            return None;
        }
        Self::BUTTONS
            .iter()
            .find(|(c, _)| u32::from(*c) == cmd.data())
            .map(|(_, b)| *b)
    }
    /// Encode a button into a command
    fn encode(button: Button) -> Option<Self::Cmd> {
        Self::BUTTONS
            .iter()
            .find(|(_, b)| *b == button)
            .and_then(|(c, _)| Self::Cmd::construct(Self::ADDRESS, *c as u32))
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
/// Extensive list of all buttons ever found on a remote control ;-)
pub enum Button {
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

    Random,
    Repeat,
    Time,
    Setup,

    PitchReset,
    PitchPlus,
    PitchMinus,
    Prog,
}
