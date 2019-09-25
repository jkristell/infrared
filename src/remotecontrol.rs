pub trait RemoteControl<CMD> {
    type Action;

    fn decode(&self, raw: CMD) -> Option<Self::Action>;

    fn encode(&self, cmd: Self::Action) -> u32;
}
