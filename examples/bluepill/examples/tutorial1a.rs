#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac,
    prelude::*,
    stm32::{interrupt, TIM2},
    timer::{CountDownTimer, Event, Timer},
};

use infrared::{
    protocol::Rc6,
    receiver::{PinInput, Poll},
    Receiver,
};

// Sample rate
const TIMER_FREQ: u32 = 20_000;

// Our receivertype
type IrReceiver = Receiver<Rc6, Poll, PinInput<PB8<Input<Floating>>>>;

// Globals
static mut TIMER: Option<CountDownTimer<TIM2>> = None;
static mut RECEIVER: Option<IrReceiver> = None;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let _core = cortex_m::Peripherals::take().unwrap();
    let device = pac::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer =
        Timer::tim2(device.TIM2, &clocks, &mut rcc.apb1).start_count_down(TIMER_FREQ.hz());

    timer.listen(Event::Update);

    let receiver = Receiver::with_pin(TIMER_FREQ, pin);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        RECEIVER.replace(receiver);
    }

    unsafe {
        // Enable the timer interrupt
        pac::NVIC::unmask(pac::Interrupt::TIM2);
    }

    rprintln!("Ready!");

    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    let receiver = unsafe { RECEIVER.as_mut().unwrap() };

    if let Ok(Some(cmd)) = receiver.poll() {
        rprintln!("Cmd: {} {}", cmd.addr, cmd.cmd);
    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();
}
