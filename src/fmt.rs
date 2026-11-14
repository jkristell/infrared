#![macro_use]
#![allow(unused_macros)]

// When both `defmt` and `log` are enabled, `defmt` takes precedence.
// Anchor the unused log dependency in that case.
#[cfg(all(feature = "log", feature = "defmt"))]
use log as _;

macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(all(feature = "log", not(feature = "defmt")))]
            ::log::trace!($s $(, $x)*);
            #[cfg(feature = "defmt")]
            ::defmt::trace!($s $(, $x)*);
            #[cfg(not(any(feature = "log", feature="defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}

macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(all(feature = "log", not(feature = "defmt")))]
            ::log::debug!($s $(, $x)*);
            #[cfg(feature = "defmt")]
            ::defmt::debug!($s $(, $x)*);
            #[cfg(not(any(feature = "log", feature="defmt")))]
            let _ = ($( & $x ),*);
        }
    };
}
