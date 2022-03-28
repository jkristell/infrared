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

    fn find<P: From<usize>>(spans: &PulseSpans<Self::Duration>, pl: Self::Duration) -> Option<P> {
        spans
            .spans
            .iter()
            .position(|v| v.contains(pl))
            .map(Into::into)
    }
}

#[derive(Debug)]
pub struct Span<T> {
    low: T,
    high: T,
}

#[derive(Debug)]
pub struct PulseSpans<T> {
    pub(crate) spans: [Span<T>; 8],
}

impl<T> Span<T>
where
    T: PartialOrd,
{
    fn contains(&self, other: T) -> bool {
        self.low <= other && other <= self.high
    }
}
