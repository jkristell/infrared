use core::marker::PhantomData;

use crate::{
    receiver::{iter::BufferIterator, time::InfraMonotonic, DecoderBuilder},
    Protocol,
};

/// Receiver that takes it input from a buffer
pub struct BufferReceiver<
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic = u32,
    Cmd: From<<Proto as Protocol>::Cmd> = <Proto as Protocol>::Cmd,
> {
    resolution: u32,
    proto: PhantomData<Proto>,
    mono: PhantomData<Mono>,
    cmd: PhantomData<Cmd>,
}

impl<Proto, Mono, Cmd> Default for BufferReceiver<Proto, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Proto, Mono, Cmd> BufferReceiver<Proto, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn new() -> Self {
        BufferReceiver {
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

    pub fn iter<'a>(
        &'a mut self,
        buf: &'a [Mono::Duration],
    ) -> BufferIterator<'a, Proto, Mono, Cmd> {
        BufferIterator::new(self.resolution, buf)
    }

    pub fn iter_with<'a, P, M, C>(
        &'a mut self,
        resolution: u32,
        buf: &'a [M::Duration],
    ) -> BufferIterator<'a, P, M, C>
    where
        P: DecoderBuilder<M>,
        M: InfraMonotonic,
        C: From<<P as Protocol>::Cmd>,
    {
        BufferIterator::new(resolution, buf)
    }
}
