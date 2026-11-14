#![no_std]
#![no_main]

use bluepill_examples as _;
use cortex_m_rt::entry;
use defmt::info;
use infrared::{protocol::*, receiver::MultiReceiver, PinReceiver};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Edge, ExtiPin, Floating, Input},
    pac::{self, interrupt, TIM2},
    prelude::*,
    rcc,
    timer::{fugit::TimerInstantU32, MonoTimerUs},
};

// Resolution of the monotonic timer
const MONO_FREQ: u32 = 1_000_000;

type Instant = TimerInstantU32<MONO_FREQ>;
type IrPin = PB8<Input<Floating>>;
type IrReceiver =
    PinReceiver<MultiReceiver<6, (SamsungNec, Rc5, Rc6, AppleNec, Nec, Denon), Instant>, IrPin>;

static mut RECEIVER: Option<IrReceiver> = None;
static mut MONO: Option<MonoTimerUs<TIM2>> = None;

#[entry]
fn main() -> ! {
    let _cp = cortex_m::Peripherals::take().unwrap();
    let mut d = pac::Peripherals::take().unwrap();

    let mut flash = d.FLASH.constrain();
    let mut rcc = d.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(48.MHz()).pclk1(24.MHz()),
        &mut flash.acr,
    );
    let mut afio = d.AFIO.constrain(&mut rcc);

    let mut gpiob = d.GPIOB.split(&mut rcc);
    let mut inpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    inpin.make_interrupt_source(&mut afio);
    inpin.trigger_on_edge(&mut d.EXTI, Edge::RisingFalling);
    inpin.enable_interrupt(&mut d.EXTI);

    let mono = d.TIM2.monotonic_us(&mut rcc);

    let receiver = PinReceiver::with_pin_multi(MONO_FREQ, inpin);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        RECEIVER.replace(receiver);
        MONO.replace(mono);
    }

    // Enable the external pin interrupt
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::EXTI9_5);
    }

    info!("Ready!");

    loop {
        continue;
    }
}

#[interrupt]
fn EXTI9_5() {
    let receiver = unsafe { RECEIVER.as_mut().unwrap() };
    let mono = unsafe { MONO.as_mut().unwrap() };

    let now = rtic_monotonic::Monotonic::now(mono);

    if let Ok(cmds) = receiver.event_instant_iter(now) {
        for cmd in cmds {
            info!("cmd: {:?}", cmd);
        }
    }

    receiver.pin_mut().clear_interrupt_pending_bit();
}
