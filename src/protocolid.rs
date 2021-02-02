#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
#[non_exhaustive]
/// Protocol id
pub enum ProtocolId {
    /// Standard Nec
    Nec = 1,
    /// Nec with 16 bit addresses
    Nec16 = 2,
    /// Nec (Samsung variant)
    NecSamsung = 3,
    /// Nec (Apple variant)
    NecApple = 4,
    /// Philips Rc5
    Rc5 = 5,
    /// Philips Rc6
    Rc6 = 6,
    /// Samsung Blu-ray player protocol
    Sbp = 7,
    /// Denon
    Denon = 8,
    /// Placeholder
    Unknown = 255,
}

impl From<u8> for ProtocolId {
    fn from(u: u8) -> Self {
        match u {
            1 => ProtocolId::Nec,
            2 => ProtocolId::Nec16,
            3 => ProtocolId::NecSamsung,
            4 => ProtocolId::NecApple,
            5 => ProtocolId::Rc5,
            6 => ProtocolId::Rc6,
            7 => ProtocolId::Sbp,
            8 => ProtocolId::Denon,
            _ => ProtocolId::Unknown,
        }
    }
}
