use core::convert::From;

use crate::remotes::Remote;

impl Remote for SpecialForMp3 {
    type Action = SpecialForMp3Action;

    fn action(&self) -> Option<Self::Action> {
        use SpecialForMp3Action::*;

        match self.cmd {
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

    fn data(&self) -> (u16, u16) {
        (0, self.cmd as u16)
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

#[derive(Debug, Clone)]
pub struct SpecialForMp3 {
    // Address is always 0 for this remote
    cmd: u8,
}

impl From<u32> for SpecialForMp3 {
    fn from(value: u32) -> Self {
        let cmd = ((value >> 16) & 0xFF) as u8;
        Self { cmd }
    }
}
