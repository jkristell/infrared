use core::marker::PhantomData;

use crate::{
    receiver::{iter::BufferIterator, time::InfraMonotonic, DecoderAdapter},
    Protocol,
};

pub struct BufferInputReceiver<
    Proto: DecoderAdapter<Mono>,
    Mono: InfraMonotonic = u32,
    Cmd: From<<Proto as Protocol>::Cmd> = <Proto as Protocol>::Cmd,
> {
    resolution: u32,
    proto: PhantomData<Proto>,
    mono: PhantomData<Mono>,
    cmd: PhantomData<Cmd>,
}

impl<Proto, Mono, Cmd> Default for BufferInputReceiver<Proto, Mono, Cmd>
where
    Proto: DecoderAdapter<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Proto, Mono, Cmd> BufferInputReceiver<Proto, Mono, Cmd>
where
    Proto: DecoderAdapter<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn new() -> Self {
        BufferInputReceiver {
            resolution: 1_000_000,
            proto: Default::default(),
            mono: Default::default(),
            cmd: Default::default(),
        }
    }

    pub fn with_resolution(resolution: u32) -> Self {
        Self {
            resolution,
            proto: Default::default(),
            mono: Default::default(),
            cmd: Default::default(),
        }
    }

    pub fn iter<'a>(&'a mut self, buf: &'a [Mono::Duration]) -> BufferIterator<Proto, Mono, Cmd> {
        BufferIterator::new(self.resolution, buf)
    }

    pub fn iter_with<'a, P, M, C>(
        &'a mut self,
        resolution: u32,
        buf: &'a [M::Duration],
    ) -> BufferIterator<P, M, C>
    where
        P: DecoderAdapter<M>,
        M: InfraMonotonic,
        C: From<<P as Protocol>::Cmd>,
    {
        BufferIterator::new(resolution, buf)
    }
}
