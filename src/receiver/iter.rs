use crate::{
    receiver::{BufferInput, DecoderState, DecoderStateMachine, Event, Receiver, Status},
    Protocol,
};

pub struct BufferIterator<'a, SM, C>
where
    SM: DecoderStateMachine<u32>,
    C: From<<SM as Protocol>::Cmd>,
{
    pub(crate) pos: usize,
    pub(crate) receiver: &'a mut Receiver<SM, Event, BufferInput<'a>, u32, C>,
}

impl<'a, SM, C> Iterator for BufferIterator<'a, SM, C>
where
    SM: DecoderStateMachine<u32>,
    C: From<<SM as Protocol>::Cmd>,
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos == self.receiver.input.0.len() {
                break None;
            }

            let pos_edge = self.pos & 0x1 == 0;
            let dt_us = self.receiver.input.0[self.pos];
            self.pos += 1;

            let state: Status = SM::new_event(
                &mut self.receiver.state,
                &self.receiver.spans,
                pos_edge,
                dt_us,
            )
            .into();

            match state {
                Status::Idle | Status::Receiving => {
                    continue;
                }
                Status::Done => {
                    let cmd = SM::command(&self.receiver.state);
                    self.receiver.state.reset();
                    break cmd.map(|r| r.into());
                }
                Status::Error(_) => {
                    self.receiver.state.reset();
                    break None;
                }
            }
        }
    }
}
