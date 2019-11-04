use crate::ProtocolId;

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum ReceiverState<CMD, ERR> {
    Idle,
    Receiving,
    Done(CMD),
    Error(ERR),
    Disabled,
}

/// Receiver trait
pub trait Receiver {
    /// The resulting command type
    type Cmd;
    /// Receive Error
    type Err;
    /// Protocol id
    const PROTOCOL_ID: ProtocolId;

    /// Sample
    fn sample(&mut self, pinval: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err>;
    /// Sample on known edge
    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err>;
    /// Reset receiver
    fn reset(&mut self);
    /// Disable receiver
    fn disable(&mut self);
}


#[cfg(feature = "embedded-hal")]
pub mod hal {
    use embedded_hal::digital::v2::InputPin;
    use crate::ReceiverState;

    /// Embedded hal receiver
    pub struct Receiver<RECV, OUTPIN> {
        recv: RECV,
        pin: OUTPIN,
    }

    impl<RECV, CMD, CMDERR, PIN, PINERR> Receiver<RECV, PIN>
        where
            RECV: crate::Receiver<Cmd = CMD, Err = CMDERR>,
            PIN: InputPin<Error =PINERR>,
    {
        pub fn new(recv: RECV, pin: PIN) -> Self {
            Self {
                pin,
                recv,
            }
        }

        pub fn destroy(self) -> PIN {
            self.pin
        }

        pub fn step(&mut self, ts: u32) -> Result<Option<CMD>, PINERR> {
            let pinval = self.pin.is_low()?;

            let res = match self.recv.sample(pinval, ts) {
                ReceiverState::Done(cmd) => {
                    self.recv.reset();
                    Some(cmd)
                },
                ReceiverState::Error(_) => {
                    self.recv.reset();
                    None
                }
                _ => None,
            };

            Ok(res)
        }
    }

    pub struct Receiver2<RECV1, RECV2, PIN> {
        recv1: RECV1,
        recv2: RECV2,
        pin: PIN,
    }

    impl<RECV1, RECV2, CMD1, CMD2, CMDERR2, CMDERR1, PIN, ERR> Receiver2<RECV1, RECV2, PIN>
        where
            RECV1: crate::Receiver<Cmd = CMD1, Err = CMDERR1>,
            RECV2: crate::Receiver<Cmd = CMD2, Err = CMDERR2>,
            PIN: InputPin<Error = ERR>,
    {
        pub fn new(recv1: RECV1, recv2: RECV2, pin: PIN) -> Self {
            Self {
                pin,
                recv1,
                recv2,
            }
        }

        pub fn step(&mut self, ts: u32) -> Result<Option<(Option<CMD1>, Option<CMD2>)>, ERR> {

            let pinval = self.pin.is_low()?;

            let res1 = match self.recv1.sample(pinval, ts) {
                ReceiverState::Done(cmd) => {
                    self.recv1.reset();
                    Some(cmd)
                }
                ReceiverState::Error(_) => {
                    self.recv1.reset();
                    None
                }
                _ => None,
            };

            let res2 = match self.recv2.sample(pinval, ts) {
                ReceiverState::Done(cmd) => {
                    self.recv2.reset();
                    Some(cmd)
                }
                ReceiverState::Error(_) => {
                    self.recv2.reset();
                    None
                }
                _ => None,
            };

            Ok(Some((res1, res2)))
        }
    }
}

#[cfg(feature = "protocol-dev")]
pub struct ReceiverDebug<STATE, EXTRA> {
    pub state: STATE,
    pub state_new: STATE,
    pub delta: u16,
    pub extra: EXTRA,
}

