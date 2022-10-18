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

pub use mapper::Button;

use crate::{cmd::AddressCommand, ProtocolId};

/// A trait describing a Remote Control
pub trait RemoteControlModel: Default {
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

impl Action {
    pub fn as_str(&self) -> &str {
        match self {
            Action::Power => "power",
            Action::Source => "",
            Action::One => "",
            Action::Two => "",
            Action::Three => "",
            Action::Four => "",
            Action::Five => "",
            Action::Six => "",
            Action::Seven => "",
            Action::Eight => "",
            Action::Nine => "",
            Action::Zero => "",
            Action::Teletext => "",
            Action::ChannelPrev => "",
            Action::VolumeUp => "volume_up",
            Action::VolumeDown => "volume_down",
            Action::VolumeMute => "",
            Action::ChannelList => "",
            Action::ChannelListNext => "",
            Action::ChannelListPrev => "",
            Action::Tools => "",
            Action::Info => "",
            Action::Return => "",
            Action::Exit => "",
            Action::Enter => "",
            Action::Up => "",
            Action::Down => "",
            Action::Left => "",
            Action::Right => "",
            Action::Red => "",
            Action::Green => "",
            Action::Yellow => "",
            Action::Blue => "",
            Action::Emanual => "",
            Action::PictureSize => "",
            Action::Subtitle => "",
            Action::Stop => "",
            Action::Rewind => "",
            Action::Play => "play",
            Action::Paus => "paus",
            Action::Play_Pause => "play_pause",
            Action::Play_Pause2 => "",
            Action::Forward => "",
            Action::Mode => "",
            Action::Shuffle => "",
            Action::U_SD => "",
            Action::Plus => "",
            Action::Minus => "",
            Action::Next => "",
            Action::Prev => "",
            Action::Eq => "",
            Action::Mute => "",
            Action::Random => "",
            Action::Repeat => "",
            Action::Time => "",
            Action::Setup => "",
            Action::Menu => "",
            Action::PitchReset => "",
            Action::PitchPlus => "",
            Action::PitchMinus => "",
            Action::Prog => "",
            Action::BatteryLow => "",
        }
    }
}