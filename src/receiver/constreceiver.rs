use crate::receiver::{
    ConstDecodeStateMachine, DecoderState, DefaultInput, Error, Event, PinInput, Poll, Status,
};

#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;

/// # ConstReceiver
///
///```
/// use infrared::{
///     receiver::Builder,
/// };
///
/// let recv = Builder::new()
///     .rc6()
///     .event_driven()
///     .build_const::<40_000>();
/// ```
///
pub struct ConstReceiver<SM, MD, IN, const R: u32>
where
    SM: ConstDecodeStateMachine<R>,
{
    state: SM::State,
    data: MD,
    input: IN,
}

impl<SM, MD, IN, const R: u32> ConstReceiver<SM, MD, IN, R>
where
    SM: ConstDecodeStateMachine<R>,
    MD: Default,
{
    pub fn with_input(input: IN) -> Self {
        ConstReceiver {
            state: SM::state(),
            data: MD::default(),
            input,
        }
    }
}

#[cfg(feature = "embedded-hal")]
impl<SM, MD, PIN, const R: u32> ConstReceiver<SM, MD, PinInput<PIN>, R>
where
    SM: ConstDecodeStateMachine<R>,
    MD: Default,
    PIN: InputPin,
{
    pub fn with_pin(pin: PIN) -> ConstReceiver<SM, MD, PinInput<PIN>, R> {
        ConstReceiver {
            state: SM::state(),
            data: MD::default(),
            input: PinInput(pin),
        }
    }
}

#[cfg(feature = "embedded-hal")]
impl<SM, PIN, const R: u32> ConstReceiver<SM, Event, PinInput<PIN>, R>
where
    SM: ConstDecodeStateMachine<R>,
    PIN: InputPin,
{
    #[inline]
    pub fn event(&mut self, dt: u32) -> Result<Option<SM::Cmd>, Error<PIN::Error>> {
        let edge = self.input.0.is_low().map_err(Error::Hal)?;
        let state: Status = SM::event(&mut self.state, dt, edge).into();

        match state {
            Status::Done => {
                let cmd = SM::command(&self.state);
                self.state.reset();
                Ok(cmd)
            }
            Status::Error(err) => {
                self.state.reset();
                Err(err.into())
            }
            Status::Idle | Status::Receiving => Ok(None),
        }
    }
    pub fn pin(&mut self) -> &mut PIN {
        &mut self.input.0
    }

    pub fn release(self) -> PIN {
        self.input.0
    }
}

impl<SM, const R: u32> ConstReceiver<SM, Event, DefaultInput, R>
where
    SM: ConstDecodeStateMachine<R>,
{
    #[inline]
    pub fn event(&mut self, dt: u32, edge: bool) -> Option<()> {
        SM::event(&mut self.state, dt, edge);
        None
    }
}

#[cfg(feature = "embedded-hal")]
impl<SM, PIN, const R: u32> ConstReceiver<SM, Poll, PinInput<PIN>, R>
where
    SM: ConstDecodeStateMachine<R>,
    PIN: InputPin,
{
    #[inline]
    pub fn poll(&mut self) -> Result<(), PIN::Error> {
        let edge = self.input.0.is_low()?;
        self.data.clock += 1;
        SM::event(&mut self.state, self.data.clock, edge);
        Ok(())
    }
}
