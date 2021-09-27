use core::ops::Range;

#[derive(Debug)]
pub struct InfraConstRange<const N: usize>(pub [Range<u32>; N]);

impl<const N: usize> InfraConstRange<N> {
    pub const fn new(vals: &[(u32, u32)], resolution: u32) -> Self {
        const EMPTY_RANGE: Range<u32> = Range { start: 0, end: 0 };
        let mut r = [EMPTY_RANGE; N];

        let mut i = 0;
        loop {
            if i == { N } || i >= vals.len() {
                break;
            }
            r[i] = infra_range(resolution, vals[i].0, vals[i].1);
            i += 1;
        }

        InfraConstRange(r)
    }

    pub fn find<T: From<usize>>(&self, dt: u32) -> Option<T> {
        self.0.iter().position(|r| r.contains(&dt)).map(T::from)
    }
}

const fn infra_range(samplerate: u32, plen: u32, percent: u32) -> Range<u32> {
    let base = scale_with_samplerate(plen, samplerate);
    let tol = (base * percent) / 100;

    Range {
        start: base - tol - 2,
        end: base + tol + 4,
    }
}

const fn scale_with_samplerate(len: u32, mut samplerate: u32) -> u32 {
    let mut div = 1_000_000;

    while len.checked_mul(samplerate).is_none() {
        div /= 1000;
        samplerate /= 1000;
    }

    (len * samplerate) / div
}

#[cfg(test)]
mod test {
    use super::{scale_with_samplerate, InfraConstRange};

    #[test]
    fn test_scale_with_samplerate() {
        let r = scale_with_samplerate(560, 20_000);
        assert_eq!(r, 11);

        let r = scale_with_samplerate(560, 100_000);
        assert_eq!(r, 56);

        let r = scale_with_samplerate(560, 1_000_000);
        assert_eq!(r, 560);
    }

    #[test]
    fn size_of_infrarange() {
        assert_eq!(core::mem::size_of::<InfraConstRange<6>>(), 6 * 4 * 2);
    }
}
