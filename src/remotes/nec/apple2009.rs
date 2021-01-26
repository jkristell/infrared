use crate::protocols::nec::NecAppleCommand;
use crate::remotecontrol::{Button, DeviceType, RemoteControl};
use crate::ProtocolId;
use Button::*;

/// Generic Mp3 used by me for testing
pub struct Apple2009;

// (page, cmd) -> Button
const BUTTONS: &[((u8, u8), Button)] = &[
    ((0x0E, 0x01), Menu),
    ((0x0E, 0x02), Play_Paus),
    ((0x0E, 0x03), Right),
    ((0x0E, 0x04), Left),
    ((0x0E, 0x05), Up),
    ((0x0E, 0x06), Down),
    ((0x00, 0x03), BatteryLow),
    ((0x00, 0x07), BatteryLow),
];

impl RemoteControl for Apple2009 {
    const MODEL: &'static str = "Apple Remote";
    const DEVTYPE: DeviceType = DeviceType::Generic;
    const PROTOCOL: ProtocolId = ProtocolId::NecApple;
    const ADDRESS: u32 = 0;
    type Cmd = NecAppleCommand;

    fn decode(cmd: NecAppleCommand) -> Option<Button> {
        BUTTONS
            .iter()
            .find(|(c, _b)| c == &(cmd.command_page, cmd.command))
            .map(|(_, b)| *b)
    }
}
