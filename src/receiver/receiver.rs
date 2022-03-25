use crate::protocol::capture::Capture;
use crate::receiver::time::{FugitMono, InfraMonotonic};
use crate::receiver::Builder;
use crate::{
    receiver::{DecoderData, DecoderStateMachine, DecodingError, Error, NoPinInput, State},
    Protocol,
};
use core::marker::PhantomData;
#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;
use fugit::TimerInstantU32;

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
/// use embedded_hal::digital::v2::InputPin;
/// use dummy_pin::DummyPin;
/// use infrared::protocol::Nec;
///
/// // -------------------------------------------
/// // Receiver setup
/// // -------------------------------------------
///
/// // The pin connected to the receiver hw
/// let input_pin = DummyPin::new_low();
///
/// // Resolution of the timer interrupt in Hz.
/// const RESOLUTION: u32 = 20_000;
///
/// let mut receiver = infrared::PeriodicPoll::<Nec, DummyPin>::with_pin(RESOLUTION, input_pin);
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
///        receiver::{NoPinInput, Builder},
///        protocol::{Rc6, Nec},
///    };
///    use dummy_pin::DummyPin;
///    use infrared::receiver::BufferInputReceiver;
///
///    // Receiver for Rc6 signals, event based with embedded-hal pin
///    let pin = DummyPin::new_low();
///    let r1: Receiver<Rc6, DummyPin> = Receiver::with_pin(40_000, pin);
///
///    // Periodic polled Nec Receiver
///    let pin = DummyPin::new_low();
///    let r2: infrared::PeriodicPoll<Nec, DummyPin> = infrared::PeriodicPoll::with_pin(40_000, pin);
///
///    let mut r3: BufferInputReceiver<Rc6> = BufferInputReceiver::with_resolution(20_000);
///
///    let buf: &[u32] = &[20, 40, 20];
///    let cmd_iter = r3.iter(buf);
///
/// ```
pub struct Receiver<
    Proto: DecoderStateMachine<Mono>,
    Input = NoPinInput,
    Mono: InfraMonotonic = u32,
    Cmd: From<<Proto as Protocol>::Cmd> = <Proto as Protocol>::Cmd,
> {
    /// Decoder data
    pub(crate) state: Proto::Data,
    /// Input
    pub(crate) input: Input,

    pub(crate) spans: PulseSpans<Mono::Duration>,

    prev_instant: Mono::Instant,

    /// Type of the final command output
    pub(crate) output: PhantomData<Cmd>,
}

impl Receiver<Capture> {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl<Proto, Mono, Cmd> Receiver<Proto, NoPinInput, Mono, Cmd>
where
    Proto: DecoderStateMachine<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn new(resolution: u32) -> Receiver<Proto, NoPinInput, Mono, Cmd> {
        let state = Proto::create_data();

        Receiver {
            state,
            input: NoPinInput {},
            spans: Mono::create_span::<Proto>(resolution),
            prev_instant: Mono::ZERO_INSTANT,
            output: PhantomData::default(),
        }
    }
}

impl<Proto, Input, Mono, Cmd> Receiver<Proto, Input, Mono, Cmd>
where
    Proto: DecoderStateMachine<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn with_input(resolution: u32, input: Input) -> Self {
        let state = Proto::create_data();

        Receiver {
            state,
            input,
            spans: Mono::create_span::<Proto>(resolution),
            prev_instant: Mono::ZERO_INSTANT,
            output: PhantomData::default(),
        }
    }

    pub fn spans(&self) -> &PulseSpans<<Mono as InfraMonotonic>::Duration> {
        &self.spans
    }

    pub fn event_edge(
        &mut self,
        dt: Mono::Duration,
        edge: bool,
    ) -> Result<Option<Proto::Cmd>, DecodingError> {
        // Update state machine
        let state: State = Proto::event(&mut self.state, &self.spans, edge, dt).into();

        trace!("dt: {:?}, edge: {} s: {:?}", dt, edge, state);

        match state {
            State::Done => {
                let cmd = Proto::command(&self.state);
                self.state.reset();
                Ok(cmd)
            }
            State::Error(err) => {
                self.state.reset();
                Err(err)
            }
            State::Idle | State::Receiving => Ok(None),
        }
    }
}

#[cfg(feature = "embedded-hal")]
impl<Proto, Pin, Mono, Cmd> Receiver<Proto, Pin, Mono, Cmd>
where
    Proto: DecoderStateMachine<Mono>,
    Pin: InputPin,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    /// Create a `Receiver` with `pin` as input
    pub fn with_pin(resolution: u32, pin: Pin) -> Self {
        Self::with_input(resolution, pin)
    }
}

#[cfg(feature = "embedded-hal")]
impl<Proto, Pin, const HZ: u32, Cmd> Receiver<Proto, Pin, TimerInstantU32<HZ>, Cmd>
    where
        Proto: DecoderStateMachine<TimerInstantU32<HZ>>,
        Pin: InputPin,
        Cmd: From<<Proto as Protocol>::Cmd>,
{
    /// Create a `Receiver` with `pin` as input
    pub fn with_fugit(pin: Pin) -> Self {
        Self::with_input(HZ, pin)
    }
}


impl<Proto, Mono, Cmd> Receiver<Proto, NoPinInput, Mono, Cmd>
where
    Proto: DecoderStateMachine<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn event(&mut self, dt: Mono::Duration, edge: bool) -> Result<Option<Cmd>, DecodingError> {
        Ok(self.event_edge(dt, edge)?.map(Into::into))
    }

    pub fn event_instant(&mut self, t: Mono::Instant, edge: bool) -> Result<Option<Cmd>, DecodingError> {

        let dt = Mono::checked_sub(t, self.prev_instant).unwrap_or(Mono::ZERO_DURATION);
        self.prev_instant = t;

        Ok(self.event_edge(dt, edge)?.map(Into::into))
    }

}

#[cfg(feature = "embedded-hal")]
impl<Proto, Pin, Mono, Cmd> Receiver<Proto, Pin, Mono, Cmd>
where
    Proto: DecoderStateMachine<Mono>,
    Pin: InputPin,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn event(&mut self, dt: Mono::Duration) -> Result<Option<Cmd>, Error<Pin::Error>> {
        let edge = self.input.is_low().map_err(Error::Hal)?;
        Ok(self.event_edge(dt, edge)?.map(Into::into))
    }

    pub fn event_instant(&mut self, t: Mono::Instant) -> Result<Option<Cmd>, Error<Pin::Error>> {
        let edge = self.input.is_low().map_err(Error::Hal)?;

        let dt = Mono::checked_sub(t, self.prev_instant).unwrap_or(Mono::ZERO_DURATION);
        self.prev_instant = t;

        Ok(self.event_edge(dt, edge)?.map(Into::into))
    }

    /// Get a reference to the Pin
    pub fn pin(&self) -> &Pin {
        &self.input
    }

    /// Get a mut ref to the Pin
    pub fn pin_mut(&mut self) -> &mut Pin {
        &mut self.input
    }

    /// Drop the receiver and release the pin
    pub fn release(self) -> Pin {
        self.input
    }
}
