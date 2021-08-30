use crate::{
    cmd::AddressCommand,
    remotecontrol::{Action, RemoteControlModel},
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Button<Rc, Cmd = <Rc as RemoteControlModel>::Cmd>
where
    Rc: RemoteControlModel,
{
    cmd: Cmd,
    action: Option<Action>,
    remote: core::marker::PhantomData<Rc>,
}

impl<Rc, Cmd> Button<Rc, Cmd>
where
    Rc: RemoteControlModel,
    Cmd: AddressCommand,
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
    Rc: RemoteControlModel<Cmd = Cmd>,
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
