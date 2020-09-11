use core::marker::PhantomData;
use core::ops::Range;

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
