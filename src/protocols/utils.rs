use core::marker::PhantomData;
use core::ops::Range;

pub(crate) struct InfraRange2(pub [Range<u32>; 2]);
pub(crate) struct InfraRange4(pub [Range<u32>; 4]);
pub(crate) struct InfraRange6(pub [Range<u32>; 6]);

impl InfraRange2 {
    pub const fn new(vals: &[(u32, u32); 2], samplerate: u32) -> InfraRange2 {
        InfraRange2([
            infra_range(samplerate, vals[0].0, vals[0].1),
            infra_range(samplerate, vals[1].0, vals[1].1),
        ])
    }

    pub fn find<T: From<u32>>(&self, dt: u32) -> Option<T> {
        self.0
            .iter()
            .position(|r| r.contains(&dt))
            .map(|v| v as u32)
            .map(T::from)
    }
}

impl InfraRange4 {
    pub const fn new(vals: &[(u32, u32); 4], samplerate: u32) -> InfraRange4 {
        InfraRange4([
            infra_range(samplerate, vals[0].0, vals[0].1),
            infra_range(samplerate, vals[1].0, vals[1].1),
            infra_range(samplerate, vals[2].0, vals[2].1),
            infra_range(samplerate, vals[3].0, vals[3].1),
        ])
    }

    pub fn find<T: From<usize>>(&self, dt: u32) -> Option<T> {
        self.0
            .iter()
            .position(|r| r.contains(&dt))
            .map(T::from)
    }
}

impl InfraRange6 {
    pub const fn new(vals: &[(u32, u32); 6], samplerate: u32) -> InfraRange6 {
        InfraRange6([
            infra_range(samplerate, vals[0].0, vals[0].1),
            infra_range(samplerate, vals[1].0, vals[1].1),
            infra_range(samplerate, vals[2].0, vals[2].1),
            infra_range(samplerate, vals[3].0, vals[3].1),
            infra_range(samplerate, vals[4].0, vals[4].1),
            infra_range(samplerate, vals[5].0, vals[5].1),
        ])
    }
    pub fn find(&self, dt: u32) -> Option<usize> {
        self.0
            .iter()
            .position(|r| r.contains(&dt))
    }
}

const fn infra_range(samplerate: u32, len: u32, percent: u32) -> Range<u32> {

    let base = (len * samplerate) / 1_000_000;
    let tol = (base * percent) / 100;

    Range {
        start: base - tol - 2,
        end: base + tol + 4,
    }
}


#[derive(Debug, Clone)]
pub struct PulseWidthRange<T> {
    r: [Range<u32>; 4],
    pd: PhantomData<T>,
}

impl<T> PulseWidthRange<T>
where
    T: Default + From<usize>,
{
    pub fn new(vals: &[(u32, u32); 4]) -> Self {
        PulseWidthRange {
            r: [
                pulserange(vals[0].0, vals[0].1),
                pulserange(vals[1].0, vals[1].1),
                pulserange(vals[2].0, vals[2].1),
                pulserange(vals[3].0, vals[3].1),
            ],
            pd: PhantomData,
        }
    }

    pub fn pulsewidth(&self, pulsewidth: u32) -> T {
        self.r
            .iter()
            .position(|r| r.contains(&pulsewidth))
            .map(T::from)
            .unwrap_or_default()
    }
}

const fn pulserange(units: u32, tolerance: u32) -> Range<u32> {
    let tol = (units * tolerance) / 100;

    Range {
        start: units - tol,
        end: units + tol,
    }
}
