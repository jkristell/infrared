use crate::remote::RemoteControl;

const ADDR: u8 = 0;

pub struct SpecialForMp3;

impl RemoteControl for SpecialForMp3 {
    type Action = SpecialForMp3Action;

    fn decode(&self, raw: u32) -> Option<SpecialForMp3Action> {
        let addr = (raw & 0xFF) as u8;

        if addr != ADDR {
            return None;
        }

        let cmd = ((raw >> 16) & 0xFF) as u8;
        to_action(cmd)
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
    ($out:ty, $t:path, $( ($x:expr, $y:expr, $z:pat) ),* ) => {

        fn to_action(val: u8) -> Option<$out>
        {
            use $t::*;
            match val {
                $($x => Some($y),)+
                _ => None,
            }
        }

        fn from_action(action: $out) -> u8 {
            use $t::*;
            match action {
                $($z => $x,)+
            }
        }
    };
}

generate_action_funcs!(
    SpecialForMp3Action,
    SpecialForMp3Action,
    (69, Power, Power),
    (70, Mode, Mode),
    (71, Mute, Mute),
    (68, Play_Paus, Play_Paus),
    (64, Prev, Prev),
    (67, Next, Next),
    (7, Eq, Eq),
    (21, Minus, Minus),
    (9, Plus, Plus),
    (22, Zero, Zero),
    (25, Shuffle, Shuffle),
    (13, U_SD, U_SD),
    (12, One, One),
    (24, Two, Two),
    (94, Three, Three),
    (8, Four, Four),
    (28, Five, Five),
    (90, Six, Six),
    (66, Seven, Seven),
    (82, Eight, Eight),
    (74, Nine, Nine)
);
