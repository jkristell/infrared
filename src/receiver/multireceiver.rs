#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;

#[cfg(feature = "denon")]
use crate::protocol::DenonCommand;
#[cfg(feature = "rc5")]
use crate::protocol::Rc5Command;
#[cfg(feature = "rc6")]
use crate::protocol::Rc6Command;
#[cfg(feature = "nec")]
use crate::protocol::{Nec16Command, NecAppleCommand, NecCommand, NecDebugCmd, NecSamsungCommand};
use crate::receiver::{time::InfraMonotonic, NoPinInput, DecoderAdapter, Receiver};

pub struct MultiReceiver<
    const N: usize,
    Receivers: ReceiverWrapper<N, Time>,
    Input,
    Time: InfraMonotonic = u32,
> {
    receivers: Receivers::Receivers,
    input: Input,
}

impl<const N: usize, Receivers: ReceiverWrapper<N, Mono>, Input, Mono: InfraMonotonic>
    MultiReceiver<N, Receivers, Input, Mono>
{
    pub fn new(res: u32, input: Input) -> Self {
        MultiReceiver {
            input,
            receivers: Receivers::make(res),
        }
    }

    pub fn event_generic(&mut self, dt: Mono::Duration, edge: bool) -> [Option<CmdEnum>; N] {
        Receivers::event(&mut self.receivers, dt, edge)
    }

    pub fn event_generic_iter(
        &mut self,
        dt: Mono::Duration,
        flank: bool,
    ) -> impl Iterator<Item = CmdEnum> {
        let arr = self.event_generic(dt, flank);
        arr.into_iter().flatten()
    }
}

#[cfg(feature = "embedded-hal")]
impl<const N: usize, Receivers, Pin: InputPin, Mono: InfraMonotonic>
    MultiReceiver<N, Receivers, Pin, Mono>
where
    Receivers: ReceiverWrapper<N, Mono>,
{
    pub fn event(&mut self, dt: Mono::Duration) -> Result<[Option<CmdEnum>; N], Pin::Error> {
        let edge = self.input.is_low()?;
        Ok(self.event_generic(dt, edge))
    }

    pub fn event_iter(
        &mut self,
        dt: Mono::Duration,
    ) -> Result<impl Iterator<Item = CmdEnum>, Pin::Error> {
        let arr = self.event(dt)?;
        Ok(arr.into_iter().flatten())
    }

    pub fn pin(&mut self) -> &mut Pin {
        &mut self.input
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

pub trait ReceiverWrapper<const N: usize, Mono: InfraMonotonic> {
    type Receivers;

    fn make(res: u32) -> Self::Receivers;

    fn event(rs: &mut Self::Receivers, dt: Mono::Duration, flank: bool) -> [Option<CmdEnum>; N];
}

impl<P1, P2, Mono: InfraMonotonic> ReceiverWrapper<2, Mono> for (P1, P2)
where
    P1: DecoderAdapter<Mono>,
    P2: DecoderAdapter<Mono>,
    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
{
    type Receivers = (
        Receiver<P1, NoPinInput, Mono>,
        Receiver<P2, NoPinInput, Mono>,
    );

    fn make(res: u32) -> Self::Receivers {
        (Receiver::new(res), Receiver::new(res))
    }

    fn event(rs: &mut Self::Receivers, dt: Mono::Duration, edge: bool) -> [Option<CmdEnum>; 2] {
        [
            rs.0.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.1.event(dt, edge).unwrap_or_default().map(Into::into),
        ]
    }
}

impl<P1, P2, P3, Mono: InfraMonotonic> ReceiverWrapper<3, Mono> for (P1, P2, P3)
where
    P1: DecoderAdapter<Mono>,
    P2: DecoderAdapter<Mono>,
    P3: DecoderAdapter<Mono>,
    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
    P3::Cmd: Into<CmdEnum>,
{
    type Receivers = (
        Receiver<P1, NoPinInput, Mono>,
        Receiver<P2, NoPinInput, Mono>,
        Receiver<P3, NoPinInput, Mono>,
    );

    fn make(res: u32) -> Self::Receivers {
        (Receiver::new(res), Receiver::new(res), Receiver::new(res))
    }

    fn event(rs: &mut Self::Receivers, dt: Mono::Duration, edge: bool) -> [Option<CmdEnum>; 3] {
        [
            rs.0.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.1.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.2.event(dt, edge).unwrap_or_default().map(Into::into),
        ]
    }
}

impl<P1, P2, P3, P4, Mono: InfraMonotonic> ReceiverWrapper<4, Mono> for (P1, P2, P3, P4)
where
    P1: DecoderAdapter<Mono>,
    P2: DecoderAdapter<Mono>,
    P3: DecoderAdapter<Mono>,
    P4: DecoderAdapter<Mono>,
    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
    P3::Cmd: Into<CmdEnum>,
    P4::Cmd: Into<CmdEnum>,
{
    type Receivers = (
        Receiver<P1, NoPinInput, Mono>,
        Receiver<P2, NoPinInput, Mono>,
        Receiver<P3, NoPinInput, Mono>,
        Receiver<P4, NoPinInput, Mono>,
    );

    fn make(res: u32) -> Self::Receivers {
        (
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
            Receiver::new(res),
        )
    }

    fn event(rs: &mut Self::Receivers, dt: Mono::Duration, edge: bool) -> [Option<CmdEnum>; 4] {
        [
            rs.0.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.1.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.2.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.3.event(dt, edge).unwrap_or_default().map(Into::into),
        ]
    }
}

impl<P1, P2, P3, P4, P5, Mono: InfraMonotonic> ReceiverWrapper<5, Mono> for (P1, P2, P3, P4, P5)
where
    P1: DecoderAdapter<Mono>,
    P2: DecoderAdapter<Mono>,
    P3: DecoderAdapter<Mono>,
    P4: DecoderAdapter<Mono>,
    P5: DecoderAdapter<Mono>,
    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
    P3::Cmd: Into<CmdEnum>,
    P4::Cmd: Into<CmdEnum>,
    P5::Cmd: Into<CmdEnum>,
{
    type Receivers = (
        Receiver<P1, NoPinInput, Mono>,
        Receiver<P2, NoPinInput, Mono>,
        Receiver<P3, NoPinInput, Mono>,
        Receiver<P4, NoPinInput, Mono>,
        Receiver<P5, NoPinInput, Mono>,
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

    fn event(rs: &mut Self::Receivers, dt: Mono::Duration, edge: bool) -> [Option<CmdEnum>; 5] {
        [
            rs.0.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.1.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.2.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.3.event(dt, edge).unwrap_or_default().map(Into::into),
            rs.4.event(dt, edge).unwrap_or_default().map(Into::into),
        ]
    }
}

impl<P1, P2, P3, P4, P5, P6, Mono: InfraMonotonic> ReceiverWrapper<6, Mono>
    for (P1, P2, P3, P4, P5, P6)
where
    P1: DecoderAdapter<Mono>,
    P2: DecoderAdapter<Mono>,
    P3: DecoderAdapter<Mono>,
    P4: DecoderAdapter<Mono>,
    P5: DecoderAdapter<Mono>,
    P6: DecoderAdapter<Mono>,

    P1::Cmd: Into<CmdEnum>,
    P2::Cmd: Into<CmdEnum>,
    P3::Cmd: Into<CmdEnum>,
    P4::Cmd: Into<CmdEnum>,
    P5::Cmd: Into<CmdEnum>,
    P6::Cmd: Into<CmdEnum>,
{
    type Receivers = (
        Receiver<P1, NoPinInput, Mono>,
        Receiver<P2, NoPinInput, Mono>,
        Receiver<P3, NoPinInput, Mono>,
        Receiver<P4, NoPinInput, Mono>,
        Receiver<P5, NoPinInput, Mono>,
        Receiver<P6, NoPinInput, Mono>,
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

    fn event(rs: &mut Self::Receivers, dt: Mono::Duration, edge: bool) -> [Option<CmdEnum>; 6] {
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
