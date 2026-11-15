use fugit::{Duration, ExtU32, ExtU64, Instant};

use crate::receiver::time::{TimeBase, Span};

impl<const NOM: u32, const DENOM: u32> TimeBase for Instant<u32, NOM, DENOM> {
    type Instant = Instant<u32, NOM, DENOM>;
    type Duration = Duration<u32, NOM, DENOM>;
    const ZERO_INSTANT: Self::Instant = Instant::<u32, NOM, DENOM>::from_ticks(0);
    const ZERO_DURATION: Self::Duration = Duration::<u32, NOM, DENOM>::from_ticks(0);

    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration> {
        a.checked_duration_since(b)
    }

    fn create_span(_freq: u32, p: u32, t: u32) -> Span<Self::Duration> {
        Span::<Self::Duration>::new(p.micros(), t)
    }

    fn to_micros(_freq: u32, d: Self::Duration) -> u64 {
        d.to_micros().into()
    }
}

impl<const NOM: u32, const DENOM: u32> Span<Duration<u32, NOM, DENOM>> {
    pub fn new(base: Duration<u32, NOM, DENOM>, tol: u32) -> Self {
        let tol = base * tol / 100;
        let low = base - tol;
        let high = base + tol;

        Span { low, high }
    }
}

impl<const NOM: u32, const DENOM: u32> TimeBase for Instant<u64, NOM, DENOM> {
    type Instant = Instant<u64, NOM, DENOM>;
    type Duration = Duration<u64, NOM, DENOM>;
    const ZERO_INSTANT: Self::Instant = Instant::<u64, NOM, DENOM>::from_ticks(0);
    const ZERO_DURATION: Self::Duration = Duration::<u64, NOM, DENOM>::from_ticks(0);

    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration> {
        a.checked_duration_since(b)
    }

    fn create_span(_freq: u32, p: u32, t: u32) -> Span<Self::Duration> {
        let p: u64 = p.into();
        Span::<Self::Duration>::new(p.micros(), t)
    }

    fn to_micros(_freq: u32, d: Self::Duration) -> u64 {
        d.to_micros()
    }
}

impl<const NOM: u32, const DENOM: u32> Span<Duration<u64, NOM, DENOM>> {
    pub fn new(base: Duration<u64, NOM, DENOM>, tol: u32) -> Self {
        let tol = base * tol / 100;
        let low = base - tol;
        let high = base + tol;

        Span { low, high }
    }
}
