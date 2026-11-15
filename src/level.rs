/// Level of the infrared signal
///
/// IR protocols are described as a sequence of marks (carrier on) and spaces
/// (carrier off). Receivers are told which level the signal changed to at
/// each edge.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Carrier on
    Mark,
    /// Carrier off, the idle state
    #[default]
    Space,
}

impl Level {
    /// Level of an active-low receiver module, such as the common TSOP and
    /// VS1838 types: a low output means mark
    pub const fn from_active_low(pin_is_low: bool) -> Level {
        if pin_is_low { Level::Mark } else { Level::Space }
    }

    /// Level of an active-high input: a high input means mark
    pub const fn from_active_high(pin_is_high: bool) -> Level {
        Level::from_active_low(pin_is_high)
    }

    /// True if this is a mark
    pub const fn is_mark(self) -> bool {
        matches!(self, Level::Mark)
    }

    /// The opposite level
    pub const fn toggle(self) -> Level {
        match self {
            Level::Mark => Level::Space,
            Level::Space => Level::Mark,
        }
    }
}
