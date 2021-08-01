#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Decode State machine error
pub enum DecodingError {
    /// Error while decoding address
    Address,
    /// Error decoding data bits
    Data,
    /// Validation Error
    Validation,
    /// Remotecontrol decode error
    RemoteControlError,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<PinErr> {
    Address,
    Data,
    Validation,
    RemoteControlError,
    Hal(PinErr),
}

impl<PinErr> From<DecodingError> for Error<PinErr> {
    fn from(derr: DecodingError) -> Error<PinErr> {
        match derr {
            DecodingError::Address => Error::Address,
            DecodingError::Data => Error::Data,
            DecodingError::Validation => Error::Validation,
            DecodingError::RemoteControlError => Error::RemoteControlError,
        }
    }
}
