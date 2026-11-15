use core::fmt;

/// Error from a decoder state machine
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum DecodingError {
    /// The address of the frame is invalid, e.g. a wrong vendor id
    Address,
    /// Unexpected timing while receiving the data bits
    Data,
    /// The frame was received but failed its checksum or parity check
    Validation,
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DecodingError::Address => "invalid address",
            DecodingError::Data => "unexpected timing in data",
            DecodingError::Validation => "checksum or parity mismatch",
        })
    }
}

impl core::error::Error for DecodingError {}

/// Error from a receiver that reads an input pin
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<PinErr> {
    /// The frame could not be decoded
    Decode(DecodingError),
    /// Reading the pin failed
    Pin(PinErr),
}

impl<PinErr> From<DecodingError> for Error<PinErr> {
    fn from(err: DecodingError) -> Error<PinErr> {
        Error::Decode(err)
    }
}

impl<PinErr: fmt::Debug> fmt::Display for Error<PinErr> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Decode(err) => write!(f, "decoding error: {err}"),
            Error::Pin(err) => write!(f, "pin error: {err:?}"),
        }
    }
}

impl<PinErr: fmt::Debug> core::error::Error for Error<PinErr> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Error::Decode(err) => Some(err),
            Error::Pin(_) => None,
        }
    }
}
