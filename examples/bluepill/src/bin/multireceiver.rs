#![no_std]
#![no_main]

use bluepill_examples as _;
use cortex_m_rt::entry;
use defmt::info;
use infrared::{protocol::*, receiver::MultiReceiver};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Edge, ExtiPin, Floating, Input},
    pac,
    prelude::*,
    stm32::interrupt,
    time::{Instant, MonoTimer},
};

type IrPin = PB8<Input<Floating>>;
type IrReceiver = MultiReceiver<6, (SamsungNec, Rc5, Rc6, AppleNec, Nec, Denon), IrPin>;

static mut RECEIVER: Option<IrReceiver> = None;
static mut MONO: Option<MonoTimer> = None;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut d = pac::Peripherals::take().unwrap();

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
    let mut inpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    inpin.make_interrupt_source(&mut afio);
    inpin.trigger_on_edge(&mut d.EXTI, Edge::RisingFalling);
    inpin.enable_interrupt(&mut d.EXTI);

    let mono = MonoTimer::new(cp.DWT, cp.DCB, clocks);
    let mono_freq = mono.frequency();

    let receiver = MultiReceiver::new(mono_freq.raw(), inpin);

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
    static mut LAST: Option<Instant> = None;

    let receiver = unsafe { RECEIVER.as_mut().unwrap() };
    let mono = unsafe { MONO.as_ref().unwrap() };

    let now = mono.now();

    if let Some(dt) = LAST.map(|i| i.elapsed()) {
        if let Ok(cmds) = receiver.event_iter(dt) {
            for cmd in cmds {
                info!("cmd: {:?}", cmd);
            }
        }
    }

    LAST.replace(now);

    receiver.pin().clear_interrupt_pending_bit();
}
