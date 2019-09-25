use crate::remotecontrol::RemoteControl;
use crate::nec::NecCommand;
use crate::nec_buttons;

const SAMSUNGTV_ADDR: u8 = 7;

#[derive(Copy, Clone)]
/// Samsung Tv Remote Control
pub struct SamsungTv;

impl RemoteControl<NecCommand> for SamsungTv {
    type Button = SamsungTvButton;

    fn decode(&self, cmd: NecCommand) -> Option<SamsungTvButton> {
        if cmd.addr != SAMSUNGTV_ADDR {
            return None;
        }
        to_button(cmd.cmd)
    }

    fn encode(&self, action: SamsungTvButton) -> NecCommand {
        let cmd = from_button(action);
        NecCommand {
            addr: SAMSUNGTV_ADDR,
            cmd: cmd,
        }
    }
}

nec_buttons!(
    SamsungTvButton, [
        (2, Power),
        (1, Source),
        (4, One),
        (5, Two),
        (6, Three),
        (8, Four),
        (9, Five),
        (10, Six),
        (12, Seven),
        (13, Eight),
        (14, Nine),
        (17, Zero),
        (44, Teletext),
        (19, ChannelPrev),
        (7, VolumeUp),
        (11, VolumeDown),
        (15, VolumeMute),
        (107, ChannelList),
        (18, ChannelListNext),
        (16, ChannelListPrev),
        (75, Tools),
        (31, Info),
        (88, Return),
        (45, Exit),
        (104, Enter),
        (96, Up),
        (97, Down),
        (101, Left),
        (98, Right),
        (108, Red),
        (20, Green),
        (21, Yellow),
        (22, Blue),
        (63, Emanual),
        (62, PictureSize),
        (37, Subtitle),
        (70, Stop),
        (69, Rewind),
        (71, Play),
        (74, Paus),
        (72, Forward),
    ]
);
