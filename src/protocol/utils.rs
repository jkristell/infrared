use core::ops::Range;

#[derive(Debug)]
pub struct InfraConstRange<const N: usize>(pub [Range<usize>; N]);

impl<const N: usize> InfraConstRange<N> {
    pub const fn new(vals: &[(usize, usize)], resolution: usize) -> Self {
        const EMPTY_RANGE: Range<usize> = Range { start: 0, end: 0 };
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

    pub fn find<T: From<usize>>(&self, dt: usize) -> Option<T> {
        self.0.iter().position(|r| r.contains(&dt)).map(T::from)
    }
}

const fn infra_range(samplerate: usize, plen: usize, percent: usize) -> Range<usize> {
    let base = scale_with_samplerate(plen, samplerate);
    let tol = (base * percent) / 100;

    Range {
        start: base - tol - 2,
        end: base + tol + 4,
    }
}

const fn scale_with_samplerate(len: usize, mut samplerate: usize) -> usize {
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
        assert_eq!(core::mem::size_of::<InfraConstRange<6>>(), 6 * 8 * 2);
    }
}
