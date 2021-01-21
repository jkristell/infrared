#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
#[non_exhaustive]
/// Protocol
pub enum Protocol {
    Nec = 1,
    Nec16 = 2,
    NecSamsung = 3,
    NecApple = 4,
    Rc5 = 5,
    Rc6 = 6,
    Sbp = 7,
    Unknown = 255,
}

impl From<u8> for Protocol {
    fn from(u: u8) -> Self {
        match u {
            1 => Protocol::Nec,
            2 => Protocol::Nec16,
            3 => Protocol::NecSamsung,
            4 => Protocol::NecApple,
            5 => Protocol::Rc5,
            6 => Protocol::Rc6,
            7 => Protocol::Sbp,
            _ => Protocol::Unknown,
        }
    }
}
