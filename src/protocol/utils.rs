pub(crate) const fn scale_with_samplerate(len: u32, mut samplerate: u32) -> u32 {
    let mut div = 1_000_000;

    while len.checked_mul(samplerate).is_none() {
        div /= 1000;
        samplerate /= 1000;
    }

    (len * samplerate) / div
}

#[cfg(test)]
mod test {
    use super::scale_with_samplerate;

    #[test]
    fn test_scale_with_samplerate() {
        let r = scale_with_samplerate(560, 20_000);
        assert_eq!(r, 11);

        let r = scale_with_samplerate(560, 100_000);
        assert_eq!(r, 56);

        let r = scale_with_samplerate(560, 1_000_000);
        assert_eq!(r, 560);
    }
}
