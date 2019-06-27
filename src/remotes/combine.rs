use core::convert::From;
use core::marker::PhantomData;

use crate::remotes::Remote;

#[derive(Clone)]
pub struct Combine<R1: Remote, R2: Remote> {
    val: u32,
    r1: PhantomData<R1>,
    r2: PhantomData<R2>,
}


impl<R1, R2> Remote for Combine<R1, R2>
where
    R1: Remote,
    R2: Remote,
{
    type Action = (Option<R1::Action>, Option<R2::Action>);

    fn action(&self) -> Option<Self::Action> {
        let r1: R1 = self.val.into();
        let r2: R2 = self.val.into();

        Some((r1.action(), r2.action()))
    }

    fn data(&self) -> (u16, u16) {
        let address = (self.val & 0xff) as u16;
        let command = (self.val >> 16) as u16;
        (address, command)
    }
}

impl<R1, R2> From<u32> for Combine<R1, R2>
where
   R1: Remote,
   R2: Remote,
{
    fn from(val: u32) -> Self {
        Self {
            val,
            r1: PhantomData,
            r2: PhantomData,
        }
    }
}

