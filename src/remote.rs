use core::convert::From;


pub trait RemoteControl {
    type Action;

    fn decode(&self, raw: u32) -> Option<Self::Action>;

    fn encode(&self, cmd: Self::Action) -> u32;
}

pub trait GenericRemoteControl: RemoteControl<Action=GenericCommands> { }

pub enum GenericCommands {
    Power,
    VolumeUp,

}


pub trait Remote: From<u32> {
    type Action;
    /// Retrieve the action
    fn action(&self) -> Option<Self::Action>;

    /// Get the address and command values
    fn data(&self) -> (u16, u16);
}


