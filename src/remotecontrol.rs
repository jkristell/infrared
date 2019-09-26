

/// A trait describing a Remote Control
pub trait RemoteControl<CMD> {
    /// The type of the buttons
    type Button;

    /// Try to decode a command into an Button for this remote
    fn decode(&self, raw: CMD) -> Option<Self::Button>;

    /// Encode a button into a command
    fn encode(&self, button: Self::Button) -> CMD;
}
