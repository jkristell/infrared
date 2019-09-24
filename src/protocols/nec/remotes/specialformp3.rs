use crate::remote::RemoteControl;
use crate::protocols::nec::NecCommand;

const ADDR: u16 = 0;

pub struct SpecialForMp3;

impl RemoteControl<NecCommand> for SpecialForMp3 {
    type Action = SpecialForMp3Action;

    fn decode(&self, raw: NecCommand) -> Option<SpecialForMp3Action> {

        if raw.addr != ADDR {
            return None;
        }

        to_action(raw.cmd as u8)
    }

    fn encode(&self, action: SpecialForMp3Action) -> u32 {
        let cmd = from_action(action);

        let addr = (ADDR as u32) | (!ADDR as u32) << 8;
        let cmd = (cmd as u32) << 16 | (!cmd as u32) << 24;

        addr | cmd
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum SpecialForMp3Action {
    Power,
    Mode,
    Mute,
    Play_Paus,
    Prev,
    Next,
    Eq,
    Minus,
    Plus,
    Zero,
    Shuffle,
    U_SD,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

macro_rules! generate_action_funcs {
    ($action:tt, $( ($val:expr, $name:tt) ),* ) => {

        fn to_action(val: u8) -> Option<$action> {
            use $action::*;
            match val {
                $($val => Some($name),)+
                _ => None,
            }
        }

        fn from_action(action: $action) -> u8 {
            use $action::*;
            match action {
                $($name => $val,)+
            }
        }
    };
}

generate_action_funcs!(
    SpecialForMp3Action,
    (69, Power),
    (70, Mode),
    (71, Mute),
    (68, Play_Paus),
    (64, Prev),
    (67, Next),
    (7, Eq),
    (21, Minus),
    (9, Plus),
    (22, Zero),
    (25, Shuffle),
    (13, U_SD),
    (12, One),
    (24, Two),
    (94, Three),
    (8, Four),
    (28, Five),
    (90, Six),
    (66, Seven),
    (82, Eight),
    (74, Nine)
);
