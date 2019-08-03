

pub trait RemoteControl {
    type Action;

    fn decode(&self, raw: u32) -> Option<Self::Action>;

    fn encode(&self, cmd: Self::Action) -> u32;
}


