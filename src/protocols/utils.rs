use core::marker::PhantomData;
use core::ops::Range;

#[derive(Debug, Clone)]
pub struct Ranges<T> {
    r: [Range<u32>; 4],
    pd: PhantomData<T>,
}

impl<T> Ranges<T>
where
    T: Default + From<usize>,
{
    pub fn new(vals: &([(u32, u32); 4])) -> Self {
        Ranges {
            r: [
                make_range(vals[0].0, vals[0].1),
                make_range(vals[1].0, vals[1].1),
                make_range(vals[2].0, vals[2].1),
                make_range(vals[3].0, vals[3].1),
            ],
            pd: PhantomData,
        }
    }

    pub fn pulsewidth(&self, samples: u32) -> T {
        for i in 0..self.r.len() {
            if self.r[i].contains(&samples) {
                return T::from(i);
            }
        }

        T::default()
    }
}

const fn make_range(units: u32, percent: u32) -> Range<u32> {
    let tol = (units * percent) / 100;
    (units - tol..units + tol)
}
