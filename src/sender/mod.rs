//! Infrared sender functionality
//!
//! Encoders turn a command into a pulse train of alternating mark (carrier on)
//! and space (carrier off) durations in microseconds, starting and ending with
//! a mark. The senders convert it to their own time base:
//!
//! - [`PulseSender`]: hardware independent core, driven by a periodic `tick`
//! - [`PwmSender`]: `PulseSender` driving an embedded-hal `SetDutyCycle` pwm
//! - [`AsyncSender`]: sends a whole command with an embedded-hal-async delay (`async` feature)

use crate::protocol::Protocol;

#[cfg(feature = "async")]
mod asynch;
mod buffer;
#[cfg(feature = "embedded-hal")]
mod hal;
mod senders;

#[cfg(feature = "async")]
pub use asynch::AsyncSender;
pub(crate) use buffer::PulseBuffer;
#[cfg(feature = "embedded-hal")]
pub use hal::PwmSender;
pub use senders::*;

/// Default pulse buffer size of the senders, large enough for every built-in protocol
pub const DEFAULT_BUFFER_SIZE: usize = 128;

/// Protocol that can be encoded into a pulse train
pub trait ProtocolEncoder: Protocol {
    /// Maximum number of pulses produced by [`encode`](ProtocolEncoder::encode)
    const MAX_PULSES: usize;

    /// Encode `cmd` into `buf` as alternating mark and space durations in
    /// microseconds, starting and ending with a mark
    ///
    /// Returns the number of pulses written. `buf` must hold at least
    /// [`MAX_PULSES`](ProtocolEncoder::MAX_PULSES) values.
    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize;
}

/// Convert a duration in microseconds to ticks of `resolution` Hz, rounded to nearest
pub(crate) const fn micros_to_ticks(us: u32, resolution: u32) -> u32 {
    ((us as u64 * resolution as u64 + 500_000) / 1_000_000) as u32
}

/// Builds a pulse train level by level
///
/// Consecutive levels of the same kind are merged, leading spaces are skipped
/// and a trailing space is dropped by [`finish`](PulseWriter::finish).
pub(crate) struct PulseWriter<'a> {
    buf: &'a mut [u32],
    len: usize,
}

#[allow(dead_code)]
impl<'a> PulseWriter<'a> {
    pub(crate) fn new(buf: &'a mut [u32]) -> Self {
        Self { buf, len: 0 }
    }

    pub(crate) fn mark(&mut self, us: u32) {
        self.level(true, us);
    }

    pub(crate) fn space(&mut self, us: u32) {
        self.level(false, us);
    }

    pub(crate) fn level(&mut self, mark: bool, us: u32) {
        // Even indices are marks
        let last_is_mark = self.len % 2 == 1;
        if self.len == 0 && !mark {
            return;
        }
        if self.len > 0 && last_is_mark == mark {
            self.buf[self.len - 1] += us;
        } else {
            self.buf[self.len] = us;
            self.len += 1;
        }
    }

    /// Pulse distance coded bits, least significant bit first
    pub(crate) fn pulse_distance(&mut self, bits: u64, count: u32, mark: u32, zero: u32, one: u32) {
        for i in 0..count {
            self.mark(mark);
            self.space(if (bits >> i) & 1 != 0 { one } else { zero });
        }
    }

    pub(crate) fn finish(self) -> usize {
        if self.len % 2 == 0 {
            self.len.saturating_sub(1)
        } else {
            self.len
        }
    }
}
