use crate::protocol::capture::Capture;
use crate::receiver::time::InfraMonotonic;
use crate::receiver::Builder;
use crate::{
    receiver::{
        iter::BufferIterator, BufferInput, DecoderState, DecoderStateMachine, DecodingError,
        DefaultInput, Error, Event, PinInput, Poll, Status,
    },
    Protocol,
};
use core::marker::PhantomData;
#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;

use super::time::PulseSpans;

/// # Receiver
///
/// ### Event / Interrupt based Receiver
///
/// Example:
/// ```
/// use infrared::{Receiver,
///     receiver::Builder, remotecontrol::rc5::CdPlayer, cmd::AddressCommand,
///     protocol::Rc5Command,
/// };
/// use dummy_pin::DummyPin;
///
/// // -------------------------------------------
/// // Receiver setup
/// // -------------------------------------------
///
/// // The pin connected to the receiver
/// let input_pin = DummyPin::new_high();
///
/// // Resolution of the clock used
/// const RESOLUTION: u32 = 1_000_000;
///
/// let mut receiver = Receiver::builder()
///     .rc5()
///     .event_driven()
///     .resolution(RESOLUTION)
///     .pin(input_pin)
///     .remotecontrol(CdPlayer)
///     .build();
///
/// // -------------------------------------------
/// // Input interrupt handler
/// // -------------------------------------------
///
/// let dt = 0; // Time since last pin flip
///
/// if let Ok(Some(button)) = receiver.event(dt) {
///     // Get the command associated with this button
///     let cmd = button.command();
///     println!(
///         "Action: {:?} - (Address, Command) = ({}, {})",
///         button.action(), cmd.address(), cmd.command()
///     );
/// }
///
/// ```
///
/// ### Polled
///
/// 1. Setup a CountDown-timer at a frequency of something like 20 kHz. How to setup the timer
/// and enable interrupts is HAL-specific but most HALs have examples showing you how to do it.
///
/// 2. Create a Polled `infrared::Receiver` with the desired Decoder state machine.
///
/// 3. Periodically call the poll method in the timer interrupt and it should give you a valid command
/// eventually
///
/// Something like this:
///
/// #### Polled example
/// ```
/// use infrared::{Receiver, receiver::Builder};
/// use embedded_hal::digital::v2::InputPin;
/// use dummy_pin::DummyPin;
///
/// // -------------------------------------------
/// // Receiver setup
/// // -------------------------------------------
///
/// // The pin connected to the receiver hw
/// let input_pin = DummyPin::new_high();
///
/// // Resolution of the timer interrupt in Hz.
/// const RESOLUTION: u32 = 20_000;
///
/// let mut receiver = Receiver::builder()
///     .rc5()
///     .polled()
///     .resolution(RESOLUTION)
///     .pin(input_pin)
///     .build();
///
/// // -------------------------------------------
/// // Timer interrupt handler
/// // -------------------------------------------
///
/// if let Ok(Some(cmd)) = receiver.poll() {
///     println!("{} {}", cmd.addr, cmd.cmd);
/// }
/// ```
///
/// ## Construction of receiver
///
/// ```
///    use infrared::{
///        Receiver,
///        receiver::{Event, Poll, DefaultInput, PinInput, BufferInput, Builder},
///        protocol::{Rc6, Nec},
///    };
///    use dummy_pin::DummyPin;
///
///    // Receiver for Rc6 signals, event based with embedded-hal pin
///    let pin = DummyPin::new_low();
///    let r1: Receiver<Rc6, Event, PinInput<DummyPin>> = Receiver::with_pin(40_000, pin);
///
///    // Periodic polled Nec Receiver
///    let r2: Receiver<Nec, Poll, DefaultInput> = Receiver::builder().nec().resolution(40_000).polled().build();
///
///    let buf = &[20, 40, 20];
///    let mut r3: Receiver<Rc6, Event, BufferInput> = Receiver::builder().rc6().buffer(buf).build();
///
///    let cmd_iter = r3.iter();
///
/// ```
pub struct Receiver<
    SM: DecoderStateMachine<Mono>,
    MD = Event,
    IN = DefaultInput,
    Mono: InfraMonotonic = u32,
    C: From<<SM as Protocol>::Cmd> = <SM as Protocol>::Cmd,
> {
    /// Decoder data
    pub(crate) state: SM::State,
    /// The Receiver Method and data
    pub(crate) data: MD,
    /// Input
    pub(crate) input: IN,

    pub(crate) spans: PulseSpans<Mono::Duration>,

    prev_instant: Mono::Instant,

    /// Type of the final command output
    pub(crate) output: PhantomData<C>,
}

impl Receiver<Capture, Event, DefaultInput> {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl<SM, MD, T, C> Receiver<SM, MD, DefaultInput, T, C>
where
    SM: DecoderStateMachine<T>,
    MD: Default,
    T: InfraMonotonic,
    C: From<<SM as Protocol>::Cmd>,
{
    pub fn new(resolution: u32) -> Receiver<SM, MD, DefaultInput, T, C> {
        let state = SM::state();
        let data = MD::default();

        Receiver {
            state,
            data,
            input: DefaultInput {},
            spans: T::create_span::<SM>(resolution),
            prev_instant: T::ZERO,
            output: PhantomData::default(),
        }
    }
}

impl<SM, MD, IN, T, C> Receiver<SM, MD, IN, T, C>
where
    SM: DecoderStateMachine<T>,
    MD: Default,
    T: InfraMonotonic,
    C: From<<SM as Protocol>::Cmd>,
{
    pub fn with_input(resolution: u32, input: IN) -> Self {
        let state = SM::state();
        let data = MD::default();

        debug!("Creating receiver");

        Receiver {
            state,
            data,
            input,
            spans: T::create_span::<SM>(resolution),
            prev_instant: T::ZERO,
            output: PhantomData::default(),
        }
    }

    pub fn spans(&self) -> &PulseSpans<<T as InfraMonotonic>::Duration> {
        &self.spans
    }

    pub fn generic_event(
        &mut self,
        dt: T::Duration,
        edge: bool,
    ) -> Result<Option<SM::Cmd>, DecodingError> {
        // Update state machine
        let state: Status = SM::new_event(&mut self.state, &self.spans, edge, dt).into();

        trace!("dt: {:?}, edge: {} s: {:?}", dt, edge, state);

        match state {
            Status::Done => {
                let cmd = SM::command(&self.state);
                self.state.reset();
                Ok(cmd)
            }
            Status::Error(err) => {
                self.state.reset();
                Err(err)
            }
            Status::Idle | Status::Receiving => Ok(None),
        }
    }
}

impl<'a, SM, MD, T, C> Receiver<SM, MD, BufferInput<'a>, T, C>
where
    SM: DecoderStateMachine<T>,
    MD: Default,
    T: InfraMonotonic,
    C: From<<SM as Protocol>::Cmd>,
{
    /// Create a Receiver with `buf` as input
    pub fn with_buffer(resolution: u32, buf: &'a [u32]) -> Self {
        Self::with_input(resolution, BufferInput(buf))
    }
}

#[cfg(feature = "embedded-hal")]
impl<SM, MD, PIN, T, C> Receiver<SM, MD, PinInput<PIN>, T, C>
where
    SM: DecoderStateMachine<T>,
    MD: Default,
    PIN: InputPin,
    T: InfraMonotonic,
    C: From<<SM as Protocol>::Cmd>,
{
    /// Create a `Receiver` with `pin` as input
    pub fn with_pin(resolution: u32, pin: PIN) -> Self {
        Self::with_input(resolution, PinInput(pin))
    }
}

impl<SM, T, C> Receiver<SM, Event, DefaultInput, T, C>
where
    SM: DecoderStateMachine<T>,
    T: InfraMonotonic,
    C: From<<SM as Protocol>::Cmd>,
{
    pub fn event(&mut self, dt: T::Duration, edge: bool) -> Result<Option<C>, DecodingError> {
        Ok(self.generic_event(dt, edge)?.map(Into::into))
    }
}

impl<'a, SM, C> Receiver<SM, Event, BufferInput<'a>, u32, C>
where
    SM: DecoderStateMachine<u32>,
    C: From<<SM as Protocol>::Cmd>,
{
    pub fn iter(&'a mut self) -> BufferIterator<SM, C> {
        BufferIterator {
            pos: 0,
            receiver: self,
        }
    }

    pub fn set_buffer(&mut self, b: &'a [u32]) {
        self.input.0 = b
    }
}

#[cfg(feature = "embedded-hal")]
impl<SM, P, T, C> Receiver<SM, Event, PinInput<P>, T, C>
where
    SM: DecoderStateMachine<T>,
    P: InputPin,
    T: InfraMonotonic,
    C: From<<SM as Protocol>::Cmd>,
{
    pub fn event(&mut self, dt: T::Duration) -> Result<Option<C>, Error<P::Error>> {
        let edge = self.input.0.is_low().map_err(Error::Hal)?;
        Ok(self.generic_event(dt, edge)?.map(Into::into))
    }

    pub fn fugit_time(&mut self, t: T::Instant) -> Result<Option<C>, Error<P::Error>> {
        let edge = self.input.0.is_low().map_err(Error::Hal)?;

        //let dt = t - self.prev_instant;

        let dt = T::checked_sub(t, self.prev_instant).unwrap_or(T::ZERO_DURATION);

        self.prev_instant = t;

        Ok(self.generic_event(dt, edge)?.map(Into::into))
    }

    pub fn pin(&mut self) -> &mut P {
        &mut self.input.0
    }

    pub fn release(self) -> P {
        self.input.0
    }
}

#[cfg(feature = "embedded-hal")]
impl<SM, P, C> Receiver<SM, Poll, PinInput<P>, u32, C>
where
    SM: DecoderStateMachine<u32>,
    P: InputPin,
    C: From<<SM as Protocol>::Cmd>,
{
    pub fn poll(&mut self) -> Result<Option<C>, Error<P::Error>> {
        let edge = self.input.0.is_low().map_err(Error::Hal)?;

        self.data.clock = self.data.clock.wrapping_add(1);

        if edge == self.data.edge {
            return Ok(None);
        }

        let ds = self.data.clock.wrapping_sub(self.data.last_edge);
        self.data.edge = edge;
        self.data.last_edge = self.data.clock;

        Ok(self.generic_event(ds, edge)?.map(Into::into))
    }
}
