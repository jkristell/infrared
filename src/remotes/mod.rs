use core::convert::From;

mod samsungtv;
mod specialformp3;
mod combine;

pub trait Remote: From<u32> {
    type Action;
    /// Retrieve the action
    fn action(&self) -> Option<Self::Action>;

    /// Get the address and command values
    fn data(&self) -> (u16, u16);
}

pub use samsungtv::SamsungTv;
pub use specialformp3::SpecialForMp3;
pub use combine::Combine;

