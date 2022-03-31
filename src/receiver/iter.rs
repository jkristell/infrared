use crate::{
    receiver::{
        time::InfraMonotonic, NoPinInput, ProtocolDecoder, ProtocolDecoderAdaptor, Receiver, State,
    },
    Protocol,
};

pub struct BufferIterator<'a, SM, Monotonic, C>
where
    SM: ProtocolDecoderAdaptor<Monotonic>,
    Monotonic: InfraMonotonic,
    C: From<<SM as Protocol>::Cmd>,
{
    pub(crate) pos: usize,
    pub(crate) buf: &'a [Monotonic::Duration],
    pub(crate) receiver: Receiver<SM, NoPinInput, Monotonic, C>,
}

impl<'a, SM, Monotonic, C> Iterator for BufferIterator<'a, SM, Monotonic, C>
where
    SM: ProtocolDecoderAdaptor<Monotonic>,
    Monotonic: InfraMonotonic,
    C: From<<SM as Protocol>::Cmd>,
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos == self.buf.len() {
                break None;
            }

            let pos_edge = self.pos & 0x1 == 0;
            let dt_us = self.buf[self.pos];
            self.pos += 1;

            let state: State = self.receiver.decoder.event(pos_edge, dt_us).into();

            match state {
                State::Idle | State::Receiving => {
                    continue;
                }
                State::Done => {
                    let cmd = self.receiver.decoder.command();
                    self.receiver.decoder.reset();
                    break cmd.map(|r| r.into());
                }
                State::Error(_) => {
                    self.receiver.decoder.reset();
                    break None;
                }
            }
        }
    }
}
