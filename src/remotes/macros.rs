#[macro_export]
macro_rules! remotecontrol_standardbutton {
    ( $rcname:tt, $protocol:path, $rcmodel:expr, $rctype:path, $rcaddr:expr, $rccmd:tt, [$( ($cmd:expr, $name:tt) ),* $(,)?] ) => {

        use crate::Command as _;

        pub struct $rcname;

        impl RemoteControl for $rcname {
            type Button = StandardButton;
            type Command = $rccmd;
            const PROTOCOL_ID: ProtocolId = $protocol;
            const ADDRESS: u16 = $rcaddr;
            const DEVICE: DeviceType = $rctype;
            const MODEL: &'static str = $rcmodel;
            const MAPPING: &'static [(u8, StandardButton)] = &[ $(($cmd, StandardButton::$name),)+ ];

            fn decode(cmd: Self::Command) -> Option<StandardButton> {

                if Self::ADDRESS != cmd.address() {
                    return None;
                }

                match cmd.command() {
                    $($cmd => Some(StandardButton::$name),)+
                    _ => None,
                }
            }

            fn encode(button: Self::Button) -> Option<Self::Command> {
                let stdcmd = match button {
                    $(StandardButton::$name => Some($cmd),)+
                    _ => None,
                };

                stdcmd.map(|cmd| $rccmd::construct(Self::ADDRESS, cmd))
            }
        }
    };
}
