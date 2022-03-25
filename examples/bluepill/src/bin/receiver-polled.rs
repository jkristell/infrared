#![no_std]
#![no_main]

use bluepill_examples as _;
use defmt::{Debug2Format, info};

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac,
    prelude::*,
    stm32::{interrupt, TIM2},
    timer::{CounterHz, Event, Timer},
};

#[allow(unused_imports)]
use infrared::{
    protocol::{Nec, NecApple},
    remotecontrol::{nec::*, rc5::*},
    remotecontrol::{Action, Button},
    Receiver,
};

// Pin connected to the receiver
type IrPin = PB8<Input<Floating>>;
type IrReceiver = infrared::PeriodicPoll<NecApple, IrPin, Button<Apple2009>>;

// Samplerate
const SAMPLERATE: u32 = 100_000;
// Our timer. Needs to be accessible in the interrupt handler.
static mut TIMER: Option<CounterHz<TIM2>> = None;
// Our Infrared receiver
static mut RECEIVER: Option<IrReceiver> = None;

#[entry]
fn main() -> ! {
    let _cp = cortex_m::Peripherals::take().unwrap();
    let d = pac::Peripherals::take().unwrap();

    let mut flash = d.FLASH.constrain();
    let rcc = d.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    let mut gpiob = d.GPIOB.split();
    let pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::new(d.TIM2, &clocks).counter_hz();

    timer.start(SAMPLERATE.Hz()).unwrap();

    timer.listen(Event::Update);

    let receiver = infrared::PeriodicPoll::with_pin(SAMPLERATE, pin);

    // Safe because the devices are only used from in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        RECEIVER.replace(receiver);
    }

    unsafe {
        // Enable the timer interrupt
        pac::NVIC::unmask(pac::Interrupt::TIM2);
    }

    info!("Init done!");

    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    let receiver = unsafe { RECEIVER.as_mut().unwrap() };

    let r = receiver.poll();

    match r {
        Ok(Some(cmd)) => {
            if let Some(button) = cmd.action() {
                match button {
                    Action::Play_Pause => info!("Play was pressed!"),
                    Action::Power => info!("Power on/off"),
                    _ => info!("{:?}", Debug2Format(&button)),
                };
            }
        }
        Ok(None) => {}
        Err(err) => info!("Err: {:?}", Debug2Format(&err)),
    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_interrupt(Event::Update);
}
