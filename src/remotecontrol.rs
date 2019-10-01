

pub enum DeviceType {
    Generic,
    TV,
    DVDPlayer,
    CDPlayer,
    BluRayPlayer,
}


/// A trait describing a Remote Control
pub trait RemoteControl<'a, CMD> {
    /// The type of the buttons
    type Button;

    /// Type of device that this remote controls
    const DEVICE: DeviceType = DeviceType::Generic;

    const NAME: &'a str = "";


    /// Try to decode a command into an Button for this remote
    fn decode(&self, raw: CMD) -> Option<Self::Button>;

    /// Encode a button into a command
    fn encode(&self, button: Self::Button) -> CMD;
}
