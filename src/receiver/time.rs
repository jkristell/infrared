use crate::protocol::utils::scale_with_samplerate;
use crate::receiver::DecoderStateMachine;
use core::ops::{Add, Sub};
use fugit::{ExtU32, TimerDurationU32, TimerInstantU32};

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

    fn create_span<D: DecoderStateMachine<Self>>(freq: u32) -> PulseSpans<Self::Duration>;

    fn find<P: From<usize>>(spans: &PulseSpans<Self::Duration>, pl: Self::Duration) -> Option<P> {
        spans
            .spans
            .iter()
            .position(|v| v.contains(pl))
            .map(Into::into)
    }
}

pub struct FugitMono<const HZ: u32>;

impl<const HZ: u32> InfraMonotonic for TimerInstantU32<HZ> {
    type Instant = TimerInstantU32<HZ>;
    type Duration = TimerDurationU32<HZ>;
    const ZERO_INSTANT: Self::Instant = TimerInstantU32::from_ticks(0);
    const ZERO_DURATION: Self::Duration = TimerDurationU32::from_ticks(0);

    /// Calc a - b
    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration> {
        a.checked_duration_since(b)
    }

    fn create_span<D: DecoderStateMachine<Self>>(_freq: u32) -> PulseSpans<Self::Duration> {
        PulseSpans {
            spans: [
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[0].micros(), D::TOLERANCE[0]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[1].micros(), D::TOLERANCE[1]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[2].micros(), D::TOLERANCE[2]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[3].micros(), D::TOLERANCE[3]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[4].micros(), D::TOLERANCE[4]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[5].micros(), D::TOLERANCE[5]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[6].micros(), D::TOLERANCE[6]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[7].micros(), D::TOLERANCE[7]),
            ],
        }
    }
}

impl<const HZ: u32> InfraMonotonic for FugitMono<HZ> {
    type Instant = TimerInstantU32<HZ>;
    type Duration = TimerDurationU32<HZ>;
    const ZERO_INSTANT: Self::Instant = TimerInstantU32::from_ticks(0);
    const ZERO_DURATION: Self::Duration = TimerDurationU32::from_ticks(0);

    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration> {
        a.checked_duration_since(b)
    }

    fn create_span<D: DecoderStateMachine<Self>>(_freq: u32) -> PulseSpans<Self::Duration> {
        PulseSpans {
            spans: [
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[0].micros(), D::TOLERANCE[0]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[1].micros(), D::TOLERANCE[1]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[2].micros(), D::TOLERANCE[2]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[3].micros(), D::TOLERANCE[3]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[4].micros(), D::TOLERANCE[4]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[5].micros(), D::TOLERANCE[5]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[6].micros(), D::TOLERANCE[6]),
                PulseSpan::<Self::Duration>::new(D::PULSE_LENGTHS[7].micros(), D::TOLERANCE[7]),
            ],
        }
    }
}

impl InfraMonotonic for u32 {
    type Instant = u32;
    type Duration = u32;
    const ZERO_INSTANT: Self::Instant = 0;
    const ZERO_DURATION: Self::Duration = 0;

    fn checked_sub(a: Self::Instant, b: Self::Instant) -> Option<Self::Duration> {
        a.checked_sub(b)
    }

    fn create_span<D: DecoderStateMachine<Self>>(freq: u32) -> PulseSpans<u32> {
        PulseSpans {
            spans: [
                PulseSpan::<u32>::new(
                    scale_with_samplerate(D::PULSE_LENGTHS[0], freq),
                    D::TOLERANCE[0],
                ),
                PulseSpan::<u32>::new(
                    scale_with_samplerate(D::PULSE_LENGTHS[1], freq),
                    D::TOLERANCE[1],
                ),
                PulseSpan::<u32>::new(
                    scale_with_samplerate(D::PULSE_LENGTHS[2], freq),
                    D::TOLERANCE[2],
                ),
                PulseSpan::<u32>::new(
                    scale_with_samplerate(D::PULSE_LENGTHS[3], freq),
                    D::TOLERANCE[3],
                ),
                PulseSpan::<u32>::new(
                    scale_with_samplerate(D::PULSE_LENGTHS[4], freq),
                    D::TOLERANCE[4],
                ),
                PulseSpan::<u32>::new(
                    scale_with_samplerate(D::PULSE_LENGTHS[5], freq),
                    D::TOLERANCE[5],
                ),
                PulseSpan::<u32>::new(
                    scale_with_samplerate(D::PULSE_LENGTHS[6], freq),
                    D::TOLERANCE[6],
                ),
                PulseSpan::<u32>::new(
                    scale_with_samplerate(D::PULSE_LENGTHS[7], freq),
                    D::TOLERANCE[7],
                ),
            ],
        }
    }
}

#[derive(Debug)]
pub struct PulseSpan<T> {
    low: T,
    high: T,
}

#[derive(Debug)]
pub struct PulseSpans<T> {
    spans: [PulseSpan<T>; 8],
}

impl PulseSpan<u32> {
    pub const fn new(base: u32, tol: u32) -> Self {
        let tol = base * tol / 100;
        let low = base - tol;
        let high = base + tol;

        PulseSpan { low, high }
    }
}

impl<T> PulseSpan<T>
where
    T: PartialOrd,
{
    fn contains(&self, other: T) -> bool {
        self.low <= other && other <= self.high
    }
}

impl<const HZ: u32> PulseSpan<TimerDurationU32<HZ>> {
    pub fn new(base: TimerDurationU32<HZ>, tol: u32) -> Self {
        let tol = base * tol / 100;
        let low = base - tol;
        let high = base + tol;

        PulseSpan { low, high }
    }
}
