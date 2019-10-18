use crate::ProtocolId;

#[derive(Debug)]
pub enum DeviceType {
    Generic,
    TV,
    DVDPlayer,
    CDPlayer,
    BluRayPlayer,
}

pub trait RemoteControlCommand {
    fn construct(addr: u16, cmd: u8) -> Self;
    fn address(&self) -> u16;
    fn command(&self) -> u8;
}

/// A trait describing a Remote Control
pub trait RemoteControl {
    /// The type of the buttons
    type Button;
    /// The type of command
    type Command: RemoteControlCommand;
    /// The IR protocol
    const PROTOCOL_ID: ProtocolId;
    /// Device adress
    const ADDR: u16;
    /// Type of device that this remote controls
    const DEVICE: DeviceType = DeviceType::Generic;
    /// Remote control model
    const MODEL: &'static str = "<NONAME>";
    /// command byte to standardbutton mapping
    const MAPPING: &'static [(u8, StandardButton)] = &[];

    /// Try to map a command into an Button for this remote
    fn decode_with_address(&self, cmd: Self::Command) -> Option<Self::Button> {
        if cmd.address() != Self::ADDR {
            return None;
        }
        self.decode(cmd.command())
    }

    /// Map `cmdnum` to button
    fn decode(&self, cmdnum: u8) -> Option<Self::Button>;

    /// Encode a button into a command
    fn encode(&self, button: Self::Button) -> Option<Self::Command>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

    Random,
    Repeat,
    Time,
    Setup,

    PitchReset,
    PitchPlus,
    PitchMinus,
    Prog,
}
