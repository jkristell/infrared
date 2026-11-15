//! Library with some example remote controls

pub mod mapper;

#[cfg(feature = "nec")]
pub mod nec;
#[cfg(feature = "rc5")]
pub mod rc5;
#[cfg(feature = "rc6")]
pub mod rc6;
#[cfg(feature = "samsung36")]
pub mod samsung36;

pub use mapper::Button;

use core::fmt;

use crate::Command;

/// A trait describing a Remote Control
pub trait RemoteControl: Default {
    /// Remote control model
    const MODEL: &'static str = "<NONAME>";
    /// Type of device that this remote controls
    const DEVICE_TYPE: DeviceType = DeviceType::Generic;
    /// Device address
    const ADDRESS: u32;
    /// The command type, which also determines the protocol
    type Cmd: Command;
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
            .and_then(|(c, _)| Self::Cmd::from_parts(Self::ADDRESS, *c))
    }
}

/// Device type that the remote control controls
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum DeviceType {
    /// Unspecified device
    Generic,
    /// Television
    Tv,
    /// DVD player
    DvdPlayer,
    /// CD player
    CdPlayer,
    /// Blu-ray player
    BluRayPlayer,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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
    Pause,
    PlayPause,
    PlayPause2,

    Forward,
    Mode,
    Shuffle,
    UsbSd,
    Plus,
    Minus,
    Next,
    Prev,
    Equalizer,
    Mute,

    Random,
    Repeat,
    Time,
    Setup,
    Menu,

    PitchReset,
    PitchPlus,
    PitchMinus,
    Program,

    BatteryLow,
}

impl Action {
    /// Name of the action in snake case, e.g. `"volume_up"`
    pub const fn as_str(&self) -> &'static str {
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
            Action::Pause => "pause",
            Action::PlayPause => "play_pause",
            Action::PlayPause2 => "play_pause2",
            Action::Forward => "forward",
            Action::Mode => "mode",
            Action::Shuffle => "shuffle",
            Action::UsbSd => "usb_sd",
            Action::Plus => "plus",
            Action::Minus => "minus",
            Action::Next => "next",
            Action::Prev => "prev",
            Action::Equalizer => "equalizer",
            Action::Mute => "mute",
            Action::Random => "random",
            Action::Repeat => "repeat",
            Action::Time => "time",
            Action::Setup => "setup",
            Action::Menu => "menu",
            Action::PitchReset => "pitch_reset",
            Action::PitchPlus => "pitch_plus",
            Action::PitchMinus => "pitch_minus",
            Action::Program => "program",
            Action::BatteryLow => "battery_low",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
