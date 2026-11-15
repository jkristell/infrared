//! NEC frames without interpretation of the bits. Useful for debugging

use crate::protocol::nec::{NecCommandVariant, NecPulseLen, NEC_STANDARD_TIMING};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// NEC frame without any interpretation of the bits, useful for debugging
pub struct NecRawCommand {
    pub bits: u32,
    pub repeat: bool,
}

impl NecCommandVariant for NecRawCommand {
    const PULSE_DISTANCE: &'static NecPulseLen = NEC_STANDARD_TIMING;

    fn validate(_bits: u32) -> bool {
        true
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        Some(NecRawCommand { bits, repeat })
    }

    fn pack(&self) -> u32 {
        self.bits
    }

    fn repeat(&self) -> bool {
        self.repeat
    }
}
