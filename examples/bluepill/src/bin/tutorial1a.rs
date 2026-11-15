#![no_std]
#![no_main]

use bluepill_examples as _;
use cortex_m_rt::entry;
use defmt::{info, warn};
use infrared::{protocol::Rc6, PolledReceiver, PinReceiver};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac::{self, interrupt, TIM2},
    prelude::*,
    rcc,
    timer::{CounterHz, Event},
};

// Sample rate
const TIMER_FREQ: u32 = 40_000;

// Our receivertype
type IrReceiver = PinReceiver<PolledReceiver<Rc6>, PB8<Input<Floating>>>;

// Globals
static mut TIMER: Option<CounterHz<TIM2>> = None;
static mut RECEIVER: Option<IrReceiver> = None;

#[entry]
fn main() -> ! {
    let _core = cortex_m::Peripherals::take().unwrap();
    let device = pac::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(48.MHz()).pclk1(24.MHz()),
        &mut flash.acr,
    );

    let mut gpiob = device.GPIOB.split(&mut rcc);
    let pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = device.TIM2.counter_hz(&mut rcc);
    timer.start(TIMER_FREQ.Hz()).unwrap();
    timer.listen(Event::Update);

    let receiver = PinReceiver::new(PolledReceiver::new(TIMER_FREQ), pin);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        RECEIVER.replace(receiver);
    }

    unsafe {
        // Enable the timer interrupt
        pac::NVIC::unmask(pac::Interrupt::TIM2);
    }

    info!("Ready!");

    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    let receiver = unsafe { RECEIVER.as_mut().unwrap() };

    match receiver.poll() {
        Ok(Some(cmd)) => info!("Cmd: {} {}", cmd.address, cmd.command),
        Ok(None) => (),
        Err(err) => warn!("Err: {:?}", err),
    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_interrupt(Event::Update);
}
