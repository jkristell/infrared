#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ProtocolId {
    Nec = 1,
    Nec16 = 2,
    NecSamsung = 3,
    Rc5 = 4,
    Rc6 = 5,
    /// Samsung 36 bit protocol
    Sbp = 6,

    Logging = 31,
}


mod protocols;
pub use protocols::*;

mod transmitter;
pub use transmitter::{TransmitterState, Transmitter};

mod receiver;
pub use receiver::{ReceiverStateMachine, ReceiverState};

#[cfg(feature = "embedded-hal")]
pub mod hal {
    //pub use crate::receiver::hal::{HalReceiver, HalReceiver2, HalReceiver3, HalReceiver4};
    pub use crate::transmitter::PwmTransmitter;
    pub use crate::receiver::ReceiverHal;
    use crate::protocols::sbp::SbpReceiver;
    use crate::{ReceiverStateMachine, ReceiverState};
    use crate::receiver::ReceiverState::Receiving;
    use crate::remotes::RemoteControl;

    pub struct GenericHalReceiver<PIN, SM> {
        sm: SM,
        pin: PIN,
        pinval: bool,
    }

    impl<CMD, PIN, SM: ReceiverStateMachine<Cmd=CMD>> GenericHalReceiver<PIN, SM> {

        pub fn new(pin: PIN, sm: SM) -> Self {
            Self {
                sm,
                pin,
                pinval: false,
            }
        }

        pub fn new_test(pin: PIN) -> Self {
            Self {
                sm: SM::for_samplerate(40_000),
                pin,
                pinval: false,
            }
        }
    }

    impl<SM, PIN, PINERR, CMD> ReceiverHal<PIN, PINERR, CMD> for GenericHalReceiver<PIN, SM>
    where
        CMD: crate::remotes::RemoteControlCommand,
        SM: ReceiverStateMachine<Cmd=CMD>,
        PIN: embedded_hal::digital::v2::InputPin<Error=PINERR>,
    {
        fn sample(&mut self, sampletime: u32) -> Result<Option<CMD>, PINERR> {
            let pinval = self.pin.is_low()?;

            if self.pinval != pinval {
                let r = self.sm.event(pinval, sampletime);

                if let ReceiverState::Done(cmd) = r {
                    self.sm.reset();
                    return Ok(Some(cmd));
                }

                if let ReceiverState::Error(_err) = r {
                    self.sm.reset();
                }

                self.pinval = pinval;
            }

            Ok(None)
        }

        #[cfg(feature = "remotes")]
        fn sample_remote<REMOTE>(&mut self, sampletime: u32) -> Result<Option<REMOTE::Button>, PINERR>
        where
            REMOTE: crate::remotes::RemoteControl<Command=CMD>,
        {
            self
                .sample(sampletime)
                .map(|opt| opt.and_then(|cmd| REMOTE::decode_with_address(cmd)))
        }

        fn disable(&mut self) {
            unimplemented!()
        }
    }
}

#[cfg(feature = "remotes")]
pub mod remotes;

#[cfg(feature = "protocol-dev")]
pub use receiver::ReceiverDebug;

pub mod prelude {
    pub use crate::ReceiverStateMachine;
    pub use crate::Transmitter;
    pub use crate::ReceiverState;
    pub use crate::TransmitterState;
    #[cfg(feature = "embedded-hal")]
    pub use crate::hal;
}

