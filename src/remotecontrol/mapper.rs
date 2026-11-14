use crate::{
    cmd::AddressedCommand,
    remotecontrol::{Action, RemoteControl},
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Button<Rc, Cmd = <Rc as RemoteControl>::Cmd>
where
    Rc: RemoteControl,
{
    cmd: Cmd,
    action: Option<Action>,
    remote: core::marker::PhantomData<Rc>,
}

impl<Rc, Cmd> Button<Rc, Cmd>
where
    Rc: RemoteControl,
    Cmd: AddressedCommand,
{
    pub fn action(&self) -> Option<Action> {
        self.action
    }

    pub fn command(&self) -> &Cmd {
        &self.cmd
    }

    pub fn is_repeat(&self) -> bool {
        self.cmd.is_repeat()
    }
}

impl<Cmd, Rc> From<Cmd> for Button<Rc, Cmd>
where
    Rc: RemoteControl<Cmd = Cmd>,
{
    fn from(cmd: Cmd) -> Self {
        let action = Rc::decode(&cmd);
        Button {
            cmd,
            remote: Default::default(),
            action,
        }
    }
}
