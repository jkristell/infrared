use crate::{
    protocol::utils::scale_with_samplerate,
    receiver::time::{TimeBase, Span},
};

impl TimeBase for u32 {
    type Instant = u32;
    type Duration = u32;
    const ZERO_INSTANT: Self::Instant = 0;
    const ZERO_DURATION: Self::Duration = 0;

    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration> {
        // Monotonic counter: an earlier looking `a` means it wrapped
        Some(a.wrapping_sub(b))
    }

    fn create_span(freq: u32, p: u32, t: u32) -> Span<Self::Duration> {
        Span::<u32>::scaled(p, freq, t)
    }

    fn to_micros(freq: u32, d: u32) -> u64 {
        u64::from(d) * 1_000_000 / u64::from(freq)
    }
}

impl TimeBase for u64 {
    type Instant = u64;
    type Duration = u64;
    const ZERO_INSTANT: Self::Instant = 0;
    const ZERO_DURATION: Self::Duration = 0;

    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration> {
        // Monotonic counter: an earlier looking `a` means it wrapped
        Some(a.wrapping_sub(b))
    }

    fn create_span(freq: u32, p: u32, t: u32) -> Span<Self::Duration> {
        Span::<u64>::scaled(p, freq, t)
    }

    fn to_micros(freq: u32, d: u64) -> u64 {
        (u128::from(d) * 1_000_000 / u128::from(freq)) as u64
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
