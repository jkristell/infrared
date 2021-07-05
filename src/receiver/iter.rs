use crate::receiver::{BufferInput, DecoderState, DecoderStateMachine, Event, Receiver, Status};

pub struct BufferIterator<'a, SM>
where
    SM: DecoderStateMachine,
{
    pub(crate) pos: usize,
    pub(crate) receiver: &'a mut Receiver<SM, Event, BufferInput<'a>>,
}

impl<'a, Protocol: DecoderStateMachine> Iterator for BufferIterator<'a, Protocol> {
    type Item = Protocol::Cmd;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos == self.receiver.input.0.len() {
                break None;
            }

            let pos_edge = self.pos & 0x1 == 0;
            let dt_us = self.receiver.input.0[self.pos];
            self.pos += 1;

            let state: Status = Protocol::event_full(
                &mut self.receiver.state,
                &self.receiver.ranges,
                pos_edge,
                dt_us,
            )
            .into();

            match state {
                Status::Idle | Status::Receiving => {
                    continue;
                }
                Status::Done => {
                    let cmd = Protocol::command(&self.receiver.state);
                    self.receiver.state.reset();
                    break cmd;
                }
                Status::Error(_) => {
                    self.receiver.state.reset();
                    break None;
                }
            }
        }
    }
}
