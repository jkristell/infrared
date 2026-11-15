use core::marker::PhantomData;

use crate::{
    receiver::{iter::BufferIterator, time::TimeBase, DecoderBuilder},
    Protocol,
};

/// Receiver that takes it input from a buffer
pub struct BufferReceiver<
    Proto: DecoderBuilder<Time>,
    Time: TimeBase = u32,
    Cmd: From<<Proto as Protocol>::Cmd> = <Proto as Protocol>::Cmd,
> {
    resolution: u32,
    proto: PhantomData<Proto>,
    mono: PhantomData<Time>,
    cmd: PhantomData<Cmd>,
}

impl<Proto, Time, Cmd> Default for BufferReceiver<Proto, Time, Cmd>
where
    Proto: DecoderBuilder<Time>,
    Time: TimeBase,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Proto, Time, Cmd> BufferReceiver<Proto, Time, Cmd>
where
    Proto: DecoderBuilder<Time>,
    Time: TimeBase,
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
    pub fn iter<'a>(&self, buf: &'a [Time::Duration]) -> BufferIterator<'a, Proto, Time, Cmd> {
        BufferIterator::new(self.resolution, buf)
    }
}
