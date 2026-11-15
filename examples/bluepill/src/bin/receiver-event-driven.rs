#![no_std]
#![no_main]

use bluepill_examples as _;
use cortex_m_rt::entry;
use defmt::info;
#[allow(unused_imports)]
use infrared::{
    protocol::{AppleNec, Nec, Rc6},
    remotecontrol::{nec::*, rc5::*},
    PinReceiver, Receiver,
};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Edge, ExtiPin, Floating, Input},
    pac::{self, interrupt, TIM2},
    prelude::*,
    rcc,
    timer::{fugit::TimerInstantU32, MonoTimerUs},
};

// Resolution of the monotonic timer
const MONO_FREQ: u32 = 1_000_000;

// Pin connected to the receiver
type IrPin = PB8<Input<Floating>>;

// Our timer. Needs to be accessible in the interrupt handler.
static mut MONO: Option<MonoTimerUs<TIM2>> = None;

// Our Infrared receiver
static mut RECEIVER: Option<PinReceiver<Receiver<AppleNec, TimerInstantU32<MONO_FREQ>>, IrPin>> =
    None;

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
    let mut pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    pin.make_interrupt_source(&mut afio);
    pin.trigger_on_edge(&mut d.EXTI, Edge::RisingFalling);
    pin.enable_interrupt(&mut d.EXTI);

    let mono = d.TIM2.monotonic_us(&mut rcc);

    info!("Monotimer f = {:?}", MONO_FREQ);

    let receiver = PinReceiver::new(Receiver::with_fugit(), pin);

    // Safe because the devices are only used from in the interrupt handler
    unsafe {
        RECEIVER.replace(receiver);
        MONO.replace(mono);
    }

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::EXTI9_5);
    }

    info!("Infrared Receiver Ready!");

    loop {
        continue;
    }
}

#[interrupt]
fn EXTI9_5() {
    let receiver = unsafe { RECEIVER.as_mut().unwrap() };
    let mono = unsafe { MONO.as_mut().unwrap() };
    let now = rtic_monotonic::Monotonic::now(mono);

    if let Ok(Some(cmd)) = receiver.event_at(now) {
        info!("cmd: {:?}", cmd);
    }

    receiver.pin_mut().clear_interrupt_pending_bit();
}
