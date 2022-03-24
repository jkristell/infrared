use core::marker::PhantomData;
use crate::{Protocol, Receiver};
use crate::receiver::DecoderStateMachine;
use crate::receiver::iter::BufferIterator;
use crate::receiver::time::{InfraMonotonic};

pub struct BufferInputReceiver<
    Proto: DecoderStateMachine<Mono>,
    Mono: InfraMonotonic = u32,
    Cmd: From<<Proto as Protocol>::Cmd> = <Proto as Protocol>::Cmd,
> {
    /// Decoder data
    //pub(crate) state: SM::State,
    /// The Receiver Method and data
    ///

    resolution: u32,

    c: PhantomData<Proto>,
    m: PhantomData<Mono>,

    //pub(crate) spans: PulseSpans<Mono::Duration>,
    /// Type of the final command output
    pub(crate) output: PhantomData<Cmd>,
}

impl<Proto, Mono, Cmd> BufferInputReceiver<Proto, Mono, Cmd>
    where
        Proto: DecoderStateMachine<Mono>,
        Mono: InfraMonotonic,
        Cmd: From<<Proto as Protocol>::Cmd>,

{
    pub fn new() -> Self {
        BufferInputReceiver {
            resolution: 1_000_000,
            c: Default::default(),
            m: Default::default(),
            output: Default::default()
        }
    }

    pub fn with_resolution(resolution: u32) -> Self {
        Self {
            resolution,
            c: Default::default(),
            m: Default::default(),
            output: Default::default()
        }
    }

    pub fn iter<'a>(&'a mut self,
                    buf: &'a[Mono::Duration]) -> BufferIterator<Proto, Mono, Cmd> {
        BufferIterator {
            pos: 0,
            buf,
            receiver: Receiver::new(self.resolution),
        }
    }

    pub fn iter_with<'a, P, M, C>(&'a mut self,
                    resolution: u32,
                    buf: &'a[M::Duration]) -> BufferIterator<P, M, C>
    where
        P: DecoderStateMachine<M>,
        M: InfraMonotonic,
        C: From<<P as Protocol>::Cmd>,
    {
        BufferIterator {
            pos: 0,
            buf,
            receiver: Receiver::new(resolution),
        }
    }

}