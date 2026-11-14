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

    /// Get an iterator over the commands decoded from the pulse buffer `buf`
    pub fn iter<'a>(&self, buf: &'a [Mono::Duration]) -> BufferIterator<'a, Proto, Mono, Cmd> {
        BufferIterator::new(self.resolution, buf)
    }
}
