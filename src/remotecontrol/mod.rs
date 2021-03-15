//! Library with some example remote controls

#[cfg(feature = "nec")]
pub mod nec;
#[cfg(feature = "rc5")]
pub mod rc5;
#[cfg(feature = "rc6")]
pub mod rc6;
#[cfg(feature = "sbp")]
pub mod sbp;

use crate::ProtocolId;

/// A trait describing a Remote Control
pub trait RemoteControl {
    /// Remote control model
    const MODEL: &'static str = "<NONAME>";
    /// Type of device that this remote controls
    const DEVTYPE: DeviceType = DeviceType::Generic;
    /// Protocol
    const PROTOCOL: ProtocolId;
    /// Device address
    const ADDRESS: u32;
    /// The type of command
    type Cmd: AsButton;
    /// command byte to standardbutton mapping
    const BUTTONS: &'static [(u32, Button)] = &[];

    /// Try to map a command into an Button for this remote
    fn decode(cmd: &Self::Cmd) -> Option<Button> {
        // Check address
        if Self::ADDRESS != cmd.address() {
            return None;
        }
        Self::BUTTONS
            .iter()
            .find(|(c, _)| *c == cmd.command())
            .map(|(_, b)| *b)
    }

    /// Encode a button into a command
    fn encode(button: &Button) -> Option<Self::Cmd> {
        Self::BUTTONS
            .iter()
            .find(|(_, b)| b == button)
            .and_then(|(c, _)| Self::Cmd::create(Self::ADDRESS, *c as u32))
    }
}

#[derive(Debug)]
/// Device type that the remote control controls
pub enum DeviceType {
    Generic,
    TV,
    DVDPlayer,
    CDPlayer,
    BluRayPlayer,
}

/// Trait that is implemented by all Commands that fit into the basic remote control button model
pub trait AsButton: Sized {
    /// Address
    fn address(&self) -> u32;
    /// Command
    fn command(&self) -> u32;
    /// Protocol
    fn protocol(&self) -> ProtocolId;
    /// Create a Command from this Button for Self
    fn create(addr: u32, cmd: u32) -> Option<Self>;
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
    Play_Pause,
    Play_Pause2,

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
    Menu,

    PitchReset,
    PitchPlus,
    PitchMinus,
    Prog,

    BatteryLow,
}
