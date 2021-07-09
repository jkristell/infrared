use crate::protocol::capture::Capture;

use crate::receiver::iter::BufferIterator;
use crate::receiver::{
    BufferInput, Builder, DecoderState, DecoderStateMachine, DecodingError, DefaultInput, Error,
    Event, PinInput, Poll, Status,
};
#[cfg(feature = "remotes")]
use crate::remotecontrol::{AsButton, Button, RemoteControl};
#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;

/// # Receiver
///
/// ### Event / Interrupt based Receiver
///
/// Example:
/// ```
/// use infrared::{Receiver};
///
/// use embedded_hal::digital::v2::InputPin;
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
/// const RESOLUTION: usize = 1_000_000;
///
/// let mut receiver = Receiver::builder()
///     .rc5()
///     .event_driven()
///     .resolution(RESOLUTION)
///     .pin(input_pin)
///     .build();
///
/// // -------------------------------------------
/// // Input interrupt handler
/// // -------------------------------------------
///
/// let dt = 0; // Time since last pin flip
///
/// if let Ok(Some(cmd)) = receiver.event(dt) {
///     println!("{} {}", cmd.addr, cmd.cmd);
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
/// use infrared::{Receiver};
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
/// const RESOLUTION: usize = 20_000;
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
///        receiver::{Event, Poll, DefaultInput, PinInput, BufferInput},
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
pub struct Receiver<SM: DecoderStateMachine = Capture, MD = Event, IN = DefaultInput> {
    /// Decoder data
    pub(crate) state: SM::State,
    /// Precalculated decoder ranges
    pub(crate) ranges: SM::RangeData,
    /// The Receiver Method and data
    data: MD,
    /// Input
    pub(crate) input: IN,
}

impl<SM, MD, IN> Receiver<SM, MD, IN>
where
    SM: DecoderStateMachine,
    MD: Default,
{
    pub fn new(resolution: usize, input: IN) -> Self {
        let state = SM::state();
        let ranges = SM::ranges(resolution);
        let data = MD::default();

        debug!("Creating receiver");

        #[cfg(feature = "defmt")]
        {
            defmt::info!("{:?}", defmt::Debug2Format(&ranges));
        }
        #[cfg(feature = "log")]
       {
            log::info!("{:?}", &ranges);
       }

        Receiver {
            state,
            ranges,
            data,
            input,
        }
    }

    pub fn ranges(&self) -> &SM::RangeData {
        &self.ranges
    }

    pub fn generic_event(
        &mut self,
        dt: usize,
        edge: bool,
    ) -> Result<Option<SM::Cmd>, DecodingError> {
        // Update state machine
        let state: Status = SM::event_full(&mut self.state, &self.ranges, edge, dt).into();

        trace!("dt: {}, edge: {} s: {:?}", dt, edge, state);

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

#[cfg(feature = "embedded-hal")]
impl<SM, MD, PIN> Receiver<SM, MD, PinInput<PIN>>
where
    SM: DecoderStateMachine,
    MD: Default,
    PIN: InputPin,
{
    pub fn with_pin(resolution: usize, pin: PIN) -> Receiver<SM, MD, PinInput<PIN>> {
        Self::new(resolution, PinInput(pin))
    }
}

impl Receiver<Capture, Event, DefaultInput> {
    pub fn builder() -> Builder<Capture, Event, DefaultInput> {
        Builder {
            proto: Default::default(),
            input: DefaultInput,
            method: Default::default(),
            resolution: 1_000_000,
        }
    }
}

impl<SM: DecoderStateMachine> Receiver<SM, Event, DefaultInput> {
    pub fn event(&mut self, dt: usize, edge: bool) -> Result<Option<SM::Cmd>, DecodingError> {
        self.generic_event(dt, edge)
    }
}

impl<'a, SM: DecoderStateMachine> Receiver<SM, Event, BufferInput<'a>> {
    pub fn iter(&'a mut self) -> BufferIterator<SM> {
        BufferIterator {
            pos: 0,
            receiver: self,
        }
    }

    pub fn set_buffer(&mut self, b: &'a [usize]) {
        self.input.0 = b
    }
}

#[cfg(feature = "embedded-hal")]
impl<SM: DecoderStateMachine, P: InputPin> Receiver<SM, Event, PinInput<P>> {
    pub fn event(&mut self, dt: usize) -> Result<Option<SM::Cmd>, Error<P::Error>> {
        let edge = self.input.0.is_low().map_err(Error::Hal)?;

        self.generic_event(dt, edge).map_err(Into::into)
    }

    #[cfg(feature = "remotes")]
    pub fn event_remotecontrol<RC: RemoteControl<Cmd = SM::Cmd>>(
        &mut self,
        dt: usize,
    ) -> Result<Option<Button>, Error<P::Error>>
    where
        SM::Cmd: AsButton,
    {
        self.event(dt).map(|cmd| cmd.as_ref().and_then(RC::decode))
    }

    pub fn pin(&mut self) -> &mut P {
        &mut self.input.0
    }

    pub fn release(self) -> P {
        self.input.0
    }
}

#[cfg(feature = "embedded-hal")]
impl<SM: DecoderStateMachine, P: InputPin> Receiver<SM, Poll, PinInput<P>> {
    pub fn poll(&mut self) -> Result<Option<SM::Cmd>, Error<P::Error>> {
        let edge = self.input.0.is_low().map_err(Error::Hal)?;

        self.data.clock = self.data.clock.wrapping_add(1);

        if edge == self.data.edge {
            return Ok(None);
        }

        let ds = self.data.clock.wrapping_sub(self.data.last_edge);
        self.data.edge = edge;
        self.data.last_edge = self.data.clock;

        self.generic_event(ds, edge).map_err(Into::into)
    }

    #[cfg(feature = "remotes")]
    pub fn poll_remotecontrol<RC: RemoteControl<Cmd = SM::Cmd>>(
        &mut self,
    ) -> Result<Option<Button>, Error<P::Error>>
    where
        SM::Cmd: AsButton,
    {
        self.poll().map(|cmd| cmd.as_ref().and_then(RC::decode))
    }
}
