use core::convert::From;


#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum SpecialForMp3 {
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

    Unknown
}

impl From<u32> for SpecialForMp3 {
    fn from(value: u32) -> Self {
        use SpecialForMp3::*;

        let cmd = ((value >> 16) & 0xFF) as u8;

        match cmd {
            69 => Power,
            70 => Mode,
            71 => Mute,
            68 => Play_Paus,
            64 => Prev,
            67 => Next,
            7 => Eq,
            21 => Minus,
            9 => Plus,
            22 => Zero,
            25 => Shuffle,
            13 => U_SD,
            12 => One,
            24 => Two,
            94 => Three,
            8 => Four,
            28 => Five,
            90 => Six,
            66 => Seven,
            82 => Eight,
            74 => Nine,
            _ => Unknown,
        }
    }
}



