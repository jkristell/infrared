use core::convert::From;

mod combine;

pub trait Remote: From<u32> {
    type Action;
    /// Retrieve the action
    fn action(&self) -> Option<Self::Action>;

    /// Get the address and command values
    fn data(&self) -> (u16, u16);
}

