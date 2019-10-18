#[macro_export]
macro_rules! remotecontrol_standardbutton {
    ( $rcname:tt, $protocol:path, $rcmodel:expr, $rctype:path, $rcaddr:expr, $rccmd:tt, [$( ($cmd:expr, $name:tt) ),* $(,)?] ) => {

        use crate::remotes::remotecontrol::RemoteControlCommand;

        pub struct $rcname;

        impl RemoteControl for $rcname {
            type Button = StandardButton;
            type Command = $rccmd;
            const PROTOCOL_ID: ProtocolId = $protocol;
            const ADDR: u16 = $rcaddr;
            const DEVICE: DeviceType = $rctype;
            const MODEL: &'static str = $rcmodel;
            const MAPPING: &'static [(u8, StandardButton)] = &[ $(($cmd, StandardButton::$name),)+ ];

            fn decode(&self, cmdid: u8) -> Option<StandardButton> {
                match cmdid {
                    $($cmd => Some(StandardButton::$name),)+
                    _ => None,
                }
            }

            fn encode(&self, button: Self::Button) -> Option<Self::Command> {
                let stdcmd = match button {
                    $(StandardButton::$name => Some($cmd),)+
                    _ => None,
                };

                stdcmd.map(|cmd| $rccmd::construct(Self::ADDR, cmd))
            }
        }
    };
}
