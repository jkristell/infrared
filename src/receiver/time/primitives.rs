use crate::protocol::utils::scale_with_samplerate;
use crate::receiver::time::{InfraMonotonic, PulseSpans, Span};
use crate::receiver::DecoderStateMachine;

impl InfraMonotonic for u32 {
    type Instant = u32;
    type Duration = u32;
    const ZERO_INSTANT: Self::Instant = 0;
    const ZERO_DURATION: Self::Duration = 0;

    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration> {
        a.checked_sub(b)
    }

    fn create_span(freq: u32, p: u32, t: u32) -> Span<Self::Duration> {
        Span::<u32>::scaled(p, freq, t)
    }
}

impl InfraMonotonic for u64 {
    type Instant = u64;
    type Duration = u64;
    const ZERO_INSTANT: Self::Instant = 0;
    const ZERO_DURATION: Self::Duration = 0;

    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration> {
        a.checked_sub(b)
    }

    fn create_span(freq: u32, p: u32, t: u32) -> Span<Self::Duration> {
        Span::<u64>::scaled(p, freq, t)
    }
}

impl Span<u32> {
    pub const fn new(base: u32, tol: u32) -> Self {
        let tol = base * tol / 100;
        let low = base - tol;
        let high = base + tol;

        Span { low, high }
    }

    pub const fn scaled(pulse: u32, freq: u32, tol: u32) -> Self {
        let base = scale_with_samplerate(pulse, freq);
        Self::new(base, tol)
    }
}

impl Span<u64> {
    pub const fn new(base: u32, tol: u32) -> Self {
        let tol = base * tol / 100;
        let low = base - tol;
        let high = base + tol;

        Span {
            low: low as u64,
            high: high as u64,
        }
    }
    pub const fn scaled(pulse: u32, freq: u32, tol: u32) -> Self {
        let base = scale_with_samplerate(pulse, freq);
        Self::new(base, tol)
    }
}
