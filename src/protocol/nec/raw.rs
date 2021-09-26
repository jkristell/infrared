//! Nec Raw variant. Useful for debugging

use crate::protocol::nec::{NecCommandVariant, NecPulseDistance, NEC_STANDARD_TIMING};

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Nec Command without parsing of bit meaning
pub struct NecDebugCmd {
    pub bits: u32,
}

impl NecCommandVariant for NecDebugCmd {
    const PULSE_DISTANCE: &'static NecPulseDistance = NEC_STANDARD_TIMING;

    fn validate(_bits: u32) -> bool {
        true
    }

    fn unpack(bits: u32, _repeat: bool) -> Option<Self> {
        Some(NecDebugCmd { bits })
    }

    fn pack(&self) -> u32 {
        self.bits
    }
}
