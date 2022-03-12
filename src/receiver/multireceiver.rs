use crate::receiver::{DecoderStateMachine, DefaultInput, Event, PinInput, Receiver};

#[cfg(feature = "denon")]
use crate::protocol::DenonCommand;
#[cfg(feature = "rc5")]
use crate::protocol::Rc5Command;
#[cfg(feature = "rc6")]
use crate::protocol::Rc6Command;
#[cfg(feature = "nec")]
use crate::protocol::{Nec16Command, NecAppleCommand, NecCommand, NecDebugCmd, NecSamsungCommand};

#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;

pub struct MultiReceiver<Receivers: ReceiverWrapper<N>, IN, const N: usize> {
    receivers: Receivers::Receivers,
    input: IN,
}

impl<Receivers: ReceiverWrapper<N>, IN, const N: usize> MultiReceiver<Receivers, IN, N> {
    pub fn new(res: u32, input: IN) -> Self {
        MultiReceiver {
            input,
            receivers: Receivers::make(res),
        }
    }

    pub fn event_generic(&mut self, dt: u32, edge: bool) -> [Option<CmdEnum>; N] {
        Receivers::event(&mut self.receivers, dt, edge)
    }

    pub fn event_generic_iter(&mut self, dt: u32, flank: bool) -> impl Iterator<Item = CmdEnum> {
        let arr = self.event_generic(dt, flank);
        core::array::IntoIter::new(arr).flat_map(|c| c)
    }
}

#[cfg(feature = "embedded-hal")]
impl<Receivers, PIN: InputPin, const N: usize> MultiReceiver<Receivers, PinInput<PIN>, N>
where
    Receivers: ReceiverWrapper<N>,
{
    pub fn event(&mut self, dt: u32) -> Result<[Option<CmdEnum>; N], PIN::Error> {
        let edge = self.input.0.is_low()?;
        Ok(self.event_generic(dt, edge))
    }

    pub fn event_iter(&mut self, dt: u32) -> Result<impl Iterator<Item = CmdEnum>, PIN::Error> {
        let arr = self.event(dt)?;
        // Keep the actual commands we got.
        // Clippy is suggesting that we use flatten here. but that doesn't produce the right result
        Ok(core::array::IntoIter::new(arr).filter_map(|c| c))
    }

    pub fn pin(&mut self) -> &mut PIN {
        &mut self.input.0
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CmdEnum {
    #[cfg(feature = "nec")]
    Nec(NecCommand),
    #[cfg(feature = "nec")]
    Nec16(Nec16Command),
    #[cfg(feature = "nec")]
    NecSamsung(NecSamsungCommand),
    #[cfg(feature = "nec")]
    NecApple(NecAppleCommand),
    #[cfg(feature = "nec")]
    NecDebug(NecDebugCmd),
    #[cfg(feature = "rc5")]
    Rc5(Rc5Command),
    #[cfg(feature = "rc6")]
    Rc6(Rc6Command),
    #[cfg(feature = "denon")]
    Denon(DenonCommand),
}

#[cfg(feature = "nec")]
impl From<NecCommand> for CmdEnum {
    fn from(cmd: NecCommand) -> CmdEnum {
        CmdEnum::Nec(cmd)
    }
}
#[cfg(feature = "nec")]
impl From<Nec16Command> for CmdEnum {
    fn from(cmd: Nec16Command) -> CmdEnum {
        CmdEnum::Nec16(cmd)
    }
}
#[cfg(feature = "nec")]
impl From<NecSamsungCommand> for CmdEnum {
    fn from(cmd: NecSamsungCommand) -> CmdEnum {
        CmdEnum::NecSamsung(cmd)
    }
}
#[cfg(feature = "nec")]
impl From<NecAppleCommand> for CmdEnum {
    fn from(cmd: NecAppleCommand) -> CmdEnum {
        CmdEnum::NecApple(cmd)
    }
}
#[cfg(feature = "nec")]
impl From<NecDebugCmd> for CmdEnum {
    fn from(cmd: NecDebugCmd) -> CmdEnum {
        CmdEnum::NecDebug(cmd)
    }
}
#[cfg(feature = "rc5")]
impl From<Rc5Command> for CmdEnum {
    fn from(cmd: Rc5Command) -> CmdEnum {
        CmdEnum::Rc5(cmd)
    }
}
#[cfg(feature = "rc6")]
impl From<Rc6Command> for CmdEnum {
    fn from(cmd: Rc6Command) -> CmdEnum {
        CmdEnum::Rc6(cmd)
    }
}
#[cfg(feature = "denon")]
impl From<DenonCommand> for CmdEnum {
    fn from(cmd: DenonCommand) -> CmdEnum {
        CmdEnum::Denon(cmd)
    }
}

pub trait ReceiverWrapper<const N: usize> {
    type Receivers;

    fn make(res: u32) -> Self::Receivers;

    fn event(rs: &mut Self::Receivers, dt: u32, flank: bool) -> [Option<CmdEnum>; N];
}

impl<P1, P2> ReceiverWrapper<2> for (P1, P2)
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
{
    type Receivers = (
        Receiver<P1, Event, DefaultInput>,
        Receiver<P2, Event, DefaultInput>,
    );

    fn make(res: u32) -> Self::Receivers {
        (Receiver::new(res), Receiver::new(res))
    }

    fn event(rs: &mut Self::Receivers, dt: u32, edge: bool) -> [Option<CmdEnum>; 2] {
        [
            rs.0.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.1.event(dt, edge).unwrap_or_default().map(Into::into),
        ]
    }
}

impl<P1, P2, P3> ReceiverWrapper<3> for (P1, P2, P3)
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
    P3: DecoderStateMachine,
    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
    P3::Cmd: Into<CmdEnum>,
{
    #[allow(clippy::type_complexity)]
    type Receivers = (
        Receiver<P1, Event, DefaultInput>,
        Receiver<P2, Event, DefaultInput>,
        Receiver<P3, Event, DefaultInput>,
    );

    fn make(res: u32) -> Self::Receivers {
        (Receiver::new(res), Receiver::new(res), Receiver::new(res))
    }

    fn event(rs: &mut Self::Receivers, dt: u32, edge: bool) -> [Option<CmdEnum>; 3] {
        [
            rs.0.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.1.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.2.event(dt, edge).unwrap_or_default().map(Into::into),
        ]
    }
}

impl<P1, P2, P3, P4> ReceiverWrapper<4> for (P1, P2, P3, P4)
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
    P3: DecoderStateMachine,
    P4: DecoderStateMachine,
    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
    P3::Cmd: Into<CmdEnum>,
    P4::Cmd: Into<CmdEnum>,
{
    #[allow(clippy::type_complexity)]
    type Receivers = (
        Receiver<P1, Event, DefaultInput>,
        Receiver<P2, Event, DefaultInput>,
        Receiver<P3, Event, DefaultInput>,
        Receiver<P4, Event, DefaultInput>,
    );

    fn make(res: u32) -> Self::Receivers {
        (
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
        )
    }

    fn event(rs: &mut Self::Receivers, dt: u32, edge: bool) -> [Option<CmdEnum>; 4] {
        [
            rs.0.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.1.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.2.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.3.event(dt, edge).unwrap_or_default().map(Into::into),
        ]
    }
}

impl<P1, P2, P3, P4, P5> ReceiverWrapper<5> for (P1, P2, P3, P4, P5)
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
    P3: DecoderStateMachine,
    P4: DecoderStateMachine,
    P5: DecoderStateMachine,
    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
    P3::Cmd: Into<CmdEnum>,
    P4::Cmd: Into<CmdEnum>,
    P5::Cmd: Into<CmdEnum>,
{
    #[allow(clippy::type_complexity)]
    type Receivers = (
        Receiver<P1, Event, DefaultInput>,
        Receiver<P2, Event, DefaultInput>,
        Receiver<P3, Event, DefaultInput>,
        Receiver<P4, Event, DefaultInput>,
        Receiver<P5, Event, DefaultInput>,
    );

    fn make(res: u32) -> Self::Receivers {
        (
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
        )
    }

    fn event(rs: &mut Self::Receivers, dt: u32, edge: bool) -> [Option<CmdEnum>; 5] {
        [
            rs.0.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.1.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.2.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.3.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.4.event(dt, edge).unwrap_or_default().map(Into::into),
        ]
    }
}

impl<P1, P2, P3, P4, P5, P6> ReceiverWrapper<6> for (P1, P2, P3, P4, P5, P6)
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
    P3: DecoderStateMachine,
    P4: DecoderStateMachine,
    P5: DecoderStateMachine,
    P6: DecoderStateMachine,
    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
    P3::Cmd: Into<CmdEnum>,
    P4::Cmd: Into<CmdEnum>,
    P5::Cmd: Into<CmdEnum>,
    P6::Cmd: Into<CmdEnum>,
{
    #[allow(clippy::type_complexity)]
    type Receivers = (
        Receiver<P1, Event, DefaultInput>,
        Receiver<P2, Event, DefaultInput>,
        Receiver<P3, Event, DefaultInput>,
        Receiver<P4, Event, DefaultInput>,
        Receiver<P5, Event, DefaultInput>,
        Receiver<P6, Event, DefaultInput>,
    );

    fn make(res: u32) -> Self::Receivers {
        (
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
        )
    }

    fn event(rs: &mut Self::Receivers, dt: u32, edge: bool) -> [Option<CmdEnum>; 6] {
        [
            rs.0.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.1.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.2.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.3.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.4.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.5.event(dt, edge).unwrap_or_default().map(Into::into),
        ]
    }
}
