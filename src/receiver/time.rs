//! Time base abstraction: the instant and duration types used by the receivers

use core::ops::{Add, Sub};

#[cfg(feature = "fugit")]
mod fgt;
mod primitives;

/// Time base of a receiver: the types used for timestamps and durations
///
/// Implemented for plain `u32` and `u64` tick counters, which may wrap
/// around, and for the fugit `TimerInstantU32` and `TimerInstantU64` types
/// (`fugit` feature). The tick rate is the receiver's resolution.
pub trait TimeBase: Sized {
    /// A point in time
    type Instant: Ord
        + Copy
        + Add<Self::Duration, Output = Self::Instant>
        + Sub<Self::Duration, Output = Self::Instant>
        + Sub<Self::Instant, Output = Self::Duration>;
    /// The time between two instants
    type Duration: PartialOrd
        + Copy
        + Add<Self::Duration, Output = Self::Duration>
        + core::fmt::Debug;

    /// The instant at tick zero
    const ZERO_INSTANT: Self::Instant;
    /// A zero length duration
    const ZERO_DURATION: Self::Duration;

    /// Time elapsed from `b` to the later instant `a`
    ///
    /// Plain integer counters are allowed to wrap around. Returns `None` if
    /// the duration can't be determined.
    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration>;

    #[doc(hidden)]
    fn create_span(resolution: u32, p: u32, t: u32) -> Span<Self::Duration>;

    /// Convert `d` to microseconds at `resolution` Hz
    fn to_micros(resolution: u32, d: Self::Duration) -> u64;
}

#[doc(hidden)]
#[derive(Debug)]
pub struct Span<Dur> {
    low: Dur,
    high: Dur,
}

/// The accepted duration ranges of the symbols of a protocol
#[derive(Debug)]
pub struct PulseSpans<Time: TimeBase> {
    pub(crate) spans: [Span<Time::Duration>; 8],
}

impl<Time> PulseSpans<Time>
where
    Time: TimeBase,
{
    /// Create the spans from nominal lengths and tolerances in µs, at `resolution` Hz
    pub fn new(resolution: u32, pulse: &[u32; 8], tolerance: &[u32; 8]) -> Self {
        PulseSpans {
            spans: [
                Time::create_span(resolution, pulse[0], tolerance[0]),
                Time::create_span(resolution, pulse[1], tolerance[1]),
                Time::create_span(resolution, pulse[2], tolerance[2]),
                Time::create_span(resolution, pulse[3], tolerance[3]),
                Time::create_span(resolution, pulse[4], tolerance[4]),
                Time::create_span(resolution, pulse[5], tolerance[5]),
                Time::create_span(resolution, pulse[6], tolerance[6]),
                Time::create_span(resolution, pulse[7], tolerance[7]),
            ],
        }
    }

    /// Index of the span containing `pl`, converted to the symbol type `P`
    pub fn get<P: From<usize>>(&self, pl: Time::Duration) -> Option<P> {
        self.spans
            .iter()
            .position(|v| v.contains(pl))
            .map(Into::into)
    }

    /// The longest duration that is part of a valid symbol
    pub fn max(&self) -> Time::Duration {
        self.spans.iter().fold(Time::ZERO_DURATION, |max, span| {
            if span.high > max { span.high } else { max }
        })
    }

    /// Time without edges after which the input is considered idle: twice
    /// the longest valid symbol
    pub fn idle_timeout(&self) -> Time::Duration {
        let max = self.max();
        max + max
    }

    /// True if any two spans overlap, which makes the protocol ambiguous at this resolution
    pub fn check_overlaps(&self) -> bool {
        for i in 0..self.spans.len() {
            for j in 0..self.spans.len() {
                if i == j {
                    continue;
                }

                if self.spans[i].overlaps(&self.spans[j]) {
                    return true;
                }
            }
        }

        false
    }
}

impl<Dur> Span<Dur>
where
    Dur: PartialOrd + Copy,
{
    fn contains(&self, other: Dur) -> bool {
        self.low <= other && other <= self.high
    }

    fn overlaps(&self, other: &Span<Dur>) -> bool {
        self.low <= other.high && other.low <= self.high
    }
}

#[cfg(test)]
mod test {
    use super::Span;

    #[test]
    fn span_overlaps() {
        let span = |low, high| Span::<u32> { low, high };

        assert!(span(10, 20).overlaps(&span(15, 25)));
        assert!(span(15, 25).overlaps(&span(10, 20)));
        // Fully contained spans overlap too
        assert!(span(10, 20).overlaps(&span(12, 18)));
        assert!(span(12, 18).overlaps(&span(10, 20)));

        assert!(!span(10, 20).overlaps(&span(21, 30)));
        assert!(!span(21, 30).overlaps(&span(10, 20)));
    }
}
