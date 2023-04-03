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
            .and_then(|(c, _)| Self::Cmd::create(Self::ADDRESS, *c))
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
    pub fn to_str(&self) -> &str {
        match self {
            Action::Power => "power",
            Action::Source => "source",
            Action::One => "one",
            Action::Two => "two",
            Action::Three => "three",
            Action::Four => "four",
            Action::Five => "five",
            Action::Six => "six",
            Action::Seven => "seven",
            Action::Eight => "eight",
            Action::Nine => "nine",
            Action::Zero => "zero",
            Action::Teletext => "teletext",
            Action::ChannelPrev => "channel_prev",
            Action::VolumeUp => "volume_up",
            Action::VolumeDown => "volume_down",
            Action::VolumeMute => "volume_mute",
            Action::ChannelList => "channellist",
            Action::ChannelListNext => "channellist_next",
            Action::ChannelListPrev => "channellist_prev",
            Action::Tools => "tools",
            Action::Info => "info",
            Action::Return => "return",
            Action::Exit => "exit",
            Action::Enter => "enter",
            Action::Up => "up",
            Action::Down => "down",
            Action::Left => "left",
            Action::Right => "right",
            Action::Red => "red",
            Action::Green => "green",
            Action::Yellow => "yellow",
            Action::Blue => "blue",
            Action::Emanual => "e_manual",
            Action::PictureSize => "picture_size",
            Action::Subtitle => "subtitle",
            Action::Stop => "stop",
            Action::Rewind => "rewind",
            Action::Play => "play",
            Action::Paus => "pause",
            Action::Play_Pause => "play_pause",
            Action::Play_Pause2 => "play_pause2",
            Action::Forward => "forward",
            Action::Mode => "mode",
            Action::Shuffle => "shuffle",
            Action::U_SD => "u_sd",
            Action::Plus => "plus",
            Action::Minus => "minus",
            Action::Next => "next",
            Action::Prev => "prev",
            Action::Eq => "equalizer",
            Action::Mute => "mute",
            Action::Random => "random",
            Action::Repeat => "repeat",
            Action::Time => "time",
            Action::Setup => "setup",
            Action::Menu => "menu",
            Action::PitchReset => "pitch_reset",
            Action::PitchPlus => "pitch_plus",
            Action::PitchMinus => "pitch_minus",
            Action::Prog => "program",
            Action::BatteryLow => "battery_low",
        }
    }
}
