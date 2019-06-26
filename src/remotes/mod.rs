mod samsungtv;
mod specialformp3;

pub enum MaybeAction<A, ADDR, CMD> {
    Action(A),
    Unknown((ADDR, CMD)),
}

pub trait Remote<T> {
    fn action(&self) -> Option<T>;
}

pub use samsungtv::SamsungTv;
pub use specialformp3::SpecialForMp3;
