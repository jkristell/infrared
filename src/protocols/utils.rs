use core::ops::Range;
use core::marker::PhantomData;

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


pub fn diff_samples(prev: u32, cur: u32) -> u32 {
    use core::{u32, i32};
    let nsamples = cur.wrapping_sub(prev);

    if nsamples <= i32::MAX as u32 {
        (nsamples as i32).abs() as u32
    }
    else {
        0
    }
}

#[cfg(test)]
mod test {
    use crate::protocols::utils::diff_samples;

    #[test]
    fn sampletime() {
        use std::{u32};

        let tests = [
            (0, 0, 0),
            (20, 10, 0),
            (10, 20, 10),
            (u32::MAX, 9, 10),
            (u32::MAX-10, 99, 110),
            (u32::MAX, (u32::MAX/2-1), (u32::MAX/2)),
            (u32::MAX-1000, (u32::MAX/2-1001), (u32::MAX/2)),
            (u32::MAX, (u32::MAX/2) as u32, 0),
        ];

        for &(t0, t1, res) in &tests {
            let r = diff_samples(t0, t1);
            assert_eq!(r, res);
        }
    }
}

