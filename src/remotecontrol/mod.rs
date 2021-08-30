//! Library with some example remote controls

pub mod mapper;

#[cfg(feature = "nec")]
pub mod nec;
#[cfg(feature = "rc5")]
pub mod rc5;
#[cfg(feature = "rc6")]
pub mod rc6;
#[cfg(feature = "sbp")]
pub mod sbp;

use crate::{cmd::AddressCommand, ProtocolId};

pub use mapper::Button;

/// A trait describing a Remote Control
pub trait RemoteControlModel {
    /// Remote control model
    const MODEL: &'static str = "<NONAME>";
    /// Type of device that this remote controls
    const DEVTYPE: DeviceType = DeviceType::Generic;
    /// Protocol
    const PROTOCOL: ProtocolId;
    /// Device address
    const ADDRESS: u32;
    /// The type of command
    type Cmd: AddressCommand;
    /// command byte to action mapping
    const BUTTONS: &'static [(u32, Action)] = &[];

    /// Try to map a command into a Action for this remote
    fn decode(cmd: &Self::Cmd) -> Option<Action> {
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
    fn encode(button: &Action) -> Option<Self::Cmd> {
        Self::BUTTONS
            .iter()
            .find(|(_, b)| b == button)
            .and_then(|(c, _)| Self::Cmd::create(Self::ADDRESS, *c as u32))
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Device type that the remote control controls
pub enum DeviceType {
    Generic,
    TV,
    DVDPlayer,
    CDPlayer,
    BluRayPlayer,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
/// Remote control actions
pub enum Action {
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
