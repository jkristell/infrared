#![no_std]
#![no_main]

use bluepill_examples as _;
use defmt::info;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac,
    prelude::*,
    stm32::{interrupt, TIM2},
    timer::{CounterHz, Event, Timer},
};

use infrared::{protocol::Rc6, PeriodicPoll};

// Sample rate
const TIMER_FREQ: u32 = 100_000;

// Our receivertype
type IrReceiver = PeriodicPoll<Rc6, PB8<Input<Floating>>>;

// Globals
static mut TIMER: Option<CounterHz<TIM2>> = None;
static mut RECEIVER: Option<IrReceiver> = None;

#[entry]
fn main() -> ! {
    let _core = cortex_m::Peripherals::take().unwrap();
    let device = pac::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let rcc = device.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    let mut gpiob = device.GPIOB.split();
    let pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::new(device.TIM2, &clocks).counter_hz();
    timer.start(TIMER_FREQ.Hz()).unwrap();
    timer.listen(Event::Update);

    let receiver = infrared::PeriodicPoll::with_pin(TIMER_FREQ, pin);

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

    if let Ok(Some(cmd)) = receiver.poll() {
        info!("Cmd: {} {}", cmd.addr, cmd.cmd);
    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_interrupt(Event::Update);
}
