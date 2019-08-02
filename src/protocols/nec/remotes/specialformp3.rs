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

    fn encode(&self, cmd: SpecialForMp3Action) -> u32 {
        let cmd = to_command(cmd);

        let addr = (ADDR as u32) | (!ADDR as u32) << 8;
        let cmd = (cmd as u32) << 16 | (!cmd as u32) << 24;

        addr | cmd
    }
}

fn to_command(action: SpecialForMp3Action) -> u8 {
    use SpecialForMp3Action::*;
    match action {
        Power => 69,
        _ => 0,
    }
}

fn to_action(raw: u8) -> Option<SpecialForMp3Action> {
    use SpecialForMp3Action::*;

    match raw {
        69 => Some(Power),
        70 => Some(Mode),
        71 => Some(Mute),
        68 => Some(Play_Paus),
        64 => Some(Prev),
        67 => Some(Next),
        7 => Some(Eq),
        21 => Some(Minus),
        9 => Some(Plus),
        22 => Some(Zero),
        25 => Some(Shuffle),
        13 => Some(U_SD),
        12 => Some(One),
        24 => Some(Two),
        94 => Some(Three),
        8 => Some(Four),
        28 => Some(Five),
        90 => Some(Six),
        66 => Some(Seven),
        82 => Some(Eight),
        74 => Some(Nine),
        _ => None,
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


