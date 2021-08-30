//! Nec Raw variant. Useful for debugging

use crate::protocol::nec::{NecCommandVariant, NecPulseDistance, NEC_STANDARD_TIMING};

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Nec Command without parsing of bit meaning
pub struct NecRawCommand {
    pub bits: u32,
}

impl NecCommandVariant for NecRawCommand {
    const PULSE_DISTANCE: &'static NecPulseDistance = NEC_STANDARD_TIMING;

    fn validate(_bits: u32) -> bool {
        true
    }

    fn unpack(bits: u32, _repeat: bool) -> Option<Self> {
        Some(NecRawCommand { bits })
    }

    fn pack(&self) -> u32 {
        self.bits
    }
}
