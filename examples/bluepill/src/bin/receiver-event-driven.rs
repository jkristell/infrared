#![no_std]
#![no_main]

use bluepill_examples as _;
use defmt::{info, Debug2Format};

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Edge, ExtiPin, Floating, Input},
    pac,
    prelude::*,
    stm32::interrupt,
    time::{Instant, MonoTimer},
};

#[allow(unused_imports)]
use infrared::{
    protocol::{Nec, NecApple, Rc6},
    remotecontrol::{nec::*, rc5::*},
    Receiver,
};

// Pin connected to the receiver
type IrPin = PB8<Input<Floating>>;

// Our timer. Needs to be accessible in the interrupt handler.
static mut MONO: Option<MonoTimer> = None;

// Our Infrared receiver
static mut RECEIVER: Option<Receiver<NecApple, IrPin>> = None;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let d = pac::Peripherals::take().unwrap();

    let mut flash = d.FLASH.constrain();
    let rcc = d.RCC.constrain();
    let mut afio = d.AFIO.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    let mut gpiob = d.GPIOB.split();
    let mut pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    pin.make_interrupt_source(&mut afio);
    pin.trigger_on_edge(&d.EXTI, Edge::RisingFalling);
    pin.enable_interrupt(&d.EXTI);

    let mono = MonoTimer::new(cp.DWT, cp.DCB, clocks);
    let mono_freq = mono.frequency();

    info!("Monotimer f = {:?}", Debug2Format(&mono_freq));

    let receiver = Receiver::with_pin(mono_freq.raw(), pin);

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
    static mut LAST: Option<Instant> = None;

    let receiver = unsafe { RECEIVER.as_mut().unwrap() };
    let mono = unsafe { MONO.as_ref().unwrap() };
    let now = mono.now();

    if let Some(dt) = LAST.map(|i| i.elapsed()) {
        if let Ok(Some(cmd)) = receiver.event(dt) {
            info!("cmd: {:?}", cmd);
        }
    }

    LAST.replace(now);

    receiver.pin_mut().clear_interrupt_pending_bit();
}
