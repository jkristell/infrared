
#[derive(Debug)]
pub enum DeviceType {
    Generic,
    TV,
    DVDPlayer,
    CDPlayer,
    BluRayPlayer,
}

pub trait RemoteControlCommand {
    fn address(&self) -> u16;
    fn command(&self) -> u8;
}

/// A trait describing a Remote Control
pub trait RemoteControl<'a, CMD: RemoteControlCommand> {
    /// The type of the buttons
    type Button;

    /// Device adress
    const ADDR: u16;

    /// Type of device that this remote controls
    const DEVICE: DeviceType = DeviceType::Generic;

    /// Remote control model
    const MODEL: &'a str = "<NONAME>";

    /// Try to decode a command into an Button for this remote
    fn decode(&self, cmd: CMD) -> Option<Self::Button> {
        if Self::ADDR == cmd.address() {
            self.decode_cmdid(cmd.command())
        } else {
            None
        }
    }

    fn decode_cmdid(&self, cmdid: u8) -> Option<Self::Button>;

    /// Encode a button into a command
    fn encode(&self, button: Self::Button) -> Option<CMD>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
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

}

