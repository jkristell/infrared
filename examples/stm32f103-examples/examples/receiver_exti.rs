#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac,
    prelude::*,
    stm32::{interrupt, TIM2},
    timer::{CountDownTimer, Event, Timer},
};

#[allow(unused_imports)]
use infrared::{
    hal::{EventReceiver, PeriodicReceiver},
    protocols::{Nec, Rc5},
    remotes::{nec::*, rc5::*},
    Button,
};
use stm32f1xx_hal::gpio::{Edge, ExtiPin};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", info);
    exit()
}

fn exit() -> ! {
    loop {
        asm::bkpt() // halt = exit probe-run
    }
}

// Pin connected to the receiver
type RecvPin = PB8<Input<Floating>>;

// Our timer. Needs to be accessible in the interrupt handler.
static mut TIMER: Option<CountDownTimer<TIM2>> = None;

// Our Infrared receiver
static mut RECEIVER: Option<EventReceiver<Rc5, RecvPin>> = None;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let _core = cortex_m::Peripherals::take().unwrap();
    let device = pac::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();
    let mut afio = device.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let mut pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    pin.make_interrupt_source(&mut afio);
    pin.trigger_on_edge(&device.EXTI, Edge::RISING_FALLING);
    pin.enable_interrupt(&device.EXTI);

    // We want the maximum timeout time
    let mut timer =
        Timer::tim2(device.TIM2, &clocks, &mut rcc.apb1).start_count_down(1.hz());

    timer.listen(Event::Update);

    let receiver = EventReceiver::new(pin, 1_000_000);

    // Safe because the devices are only used from in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        RECEIVER.replace(receiver);
    }

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::EXTI9_5);
    }

    rprintln!("Ready!");

    loop {
        continue;
    }
}

#[interrupt]
fn EXTI9_5() {
    let timer = unsafe { TIMER.as_mut().unwrap() };
    let receiver = unsafe { RECEIVER.as_mut().unwrap() };

    receiver.pin.clear_interrupt_pending_bit();

    let dt = timer.micros_since();

    timer.reset();
    if let Ok(Some(cmd)) = receiver.edge_event(dt) {
        rprintln!("cmd: {}", cmd.cmd);
    }
}
