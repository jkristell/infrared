use core::ops::{Add, Sub};

mod fgt;
mod primitives;

pub trait InfraMonotonic: Sized {
    type Instant: Ord
        + Copy
        + Add<Self::Duration, Output = Self::Instant>
        + Sub<Self::Duration, Output = Self::Instant>
        + Sub<Self::Instant, Output = Self::Duration>;
    type Duration: PartialOrd
        + Copy
        + Add<Self::Duration, Output = Self::Duration>
        + core::fmt::Debug;

    const ZERO_INSTANT: Self::Instant;
    const ZERO_DURATION: Self::Duration;

    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration>;

    fn create_span(freq: u32, p: u32, t: u32) -> Span<Self::Duration>;
}

#[derive(Debug)]
pub struct Span<Dur> {
    low: Dur,
    high: Dur,
}

#[derive(Debug)]
pub struct PulseSpans<Dur> {
    pub(crate) spans: [Span<Dur>; 8],
}

impl<Dur> PulseSpans<Dur>
    where
        Dur: PartialOrd + Copy,
{
    pub fn get<P: From<usize>>(&self, pl: Dur) -> Option<P> {
        self
            .spans
            .iter()
            .position(|v| v.contains(pl))
            .map(Into::into)
    }

    pub fn check_overlaps(&self) -> bool {
        false
    }
}

impl<Dur> Span<Dur>
where
    Dur: PartialOrd,
{
    fn contains(&self, other: Dur) -> bool {
        self.low <= other && other <= self.high
    }
}
