#![no_std]
#![no_main]
#![allow(deprecated)]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac,
    prelude::*,
    stm32::{interrupt, TIM2},
    timer::{Event, Timer},
};

use heapless::consts::*;
use heapless::spsc::Queue;

use infrared::{
    Receiver, ReceiverState,
    nec::*,
    rc5::*,
    rc6::*,
};

const FREQ: u32 = 40_000;

static mut TIMER: Option<Timer<TIM2>> = None;
static mut IRPIN: Option<PB8<Input<Floating>>> = None;

static mut NEC: Option<NecReceiver> = None;
static mut NES: Option<NecSamsungReceiver> = None;
static mut RC5: Option<Rc5Receiver> = None;
static mut RC6: Option<Rc6Receiver> = None;

// Command Queues
static mut NECQ: Option<Queue<NecCommand, U8>> = None;
static mut RC5Q: Option<Queue<Rc5Command, U8>> = None;
static mut RC6Q: Option<Queue<Rc6Command, U8>> = None;

#[entry]
fn main() -> ! {
    let mut core = cortex_m::Peripherals::take().unwrap();
    let device = pac::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let irpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::tim2(device.TIM2, FREQ.hz(), clocks, &mut rcc.apb1);
    timer.listen(Event::Update);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        IRPIN.replace(irpin);

        NEC.replace(NecReceiver::new(FREQ));
        NES.replace(NecSamsungReceiver::new(FREQ));
        RC5.replace(Rc5Receiver::new(FREQ));
        RC6.replace(Rc6Receiver::new(FREQ));
    }

    // Initialize the queues
    unsafe {
        NECQ.replace(Queue::new());
        RC5Q.replace(Queue::new());
        RC6Q.replace(Queue::new());
    };

    let mut necq = unsafe { NECQ.as_mut().unwrap().split().1 };
    let mut rc5q = unsafe { RC5Q.as_mut().unwrap().split().1 };
    let mut rc6q = unsafe { RC6Q.as_mut().unwrap().split().1 };

    // Enable the timer interrupt
    core.NVIC.enable(pac::Interrupt::TIM2);

    hprintln!("Ready!").unwrap();

    loop {
        if let Some(cmd) = necq.dequeue() {
            hprintln!("{:?}", cmd).unwrap();
        }

        if let Some(cmd) = rc5q.dequeue() {
            hprintln!("{:?}", cmd).unwrap();
        }

        if let Some(cmd) = rc6q.dequeue() {
            hprintln!("{:?}", cmd).unwrap();
        }
    }
}

#[interrupt]
fn TIM2() {
    static mut COUNT: u32 = 0;
    static mut PINVAL: bool = false;

    // Clear the interrupt
    let timer = unsafe { &mut TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();

    // Read the value of the pin (active low)
    let new_pinval = unsafe { IRPIN.as_ref().unwrap().is_low() };

    if *PINVAL != new_pinval {
        let rising = new_pinval;

        let mut necq = unsafe { NECQ.as_mut().unwrap().split().0 };
        let mut rc5q = unsafe { RC5Q.as_mut().unwrap().split().0 };
        let mut rc6q = unsafe { RC6Q.as_mut().unwrap().split().0 };

        let nec = unsafe { NEC.as_mut().unwrap() };
        let nes = unsafe { NES.as_mut().unwrap() };
        let rc5 = unsafe { RC5.as_mut().unwrap() };
        let rc6 = unsafe { RC6.as_mut().unwrap() };

        // NEC
        match nec.sample_edge(rising, *COUNT) {
            ReceiverState::Done(cmd) => {
                necq.enqueue(cmd).ok().unwrap();
                nec.reset();
            },
            ReceiverState::Error(_err) => nec.reset(),
            _ => (),
        }

        // Nec with Samsung flavour
        match nes.sample_edge(rising, *COUNT) {
            ReceiverState::Done(cmd) => {
                necq.enqueue(cmd).unwrap();
                nes.reset();
            },
            ReceiverState::Error(_err) => nes.reset(),
            _ => (),
        }

        match rc5.sample_edge(rising, *COUNT) {
            ReceiverState::Done(cmd) => {
                rc5q.enqueue(cmd).unwrap();
                rc5.reset();
            },
            ReceiverState::Error(_err) => rc5.reset(),
            _ => (),
        }

        // RC6
        match rc6.sample_edge(rising, *COUNT) {
            ReceiverState::Done(cmd) => {
                rc6q.enqueue(cmd).unwrap();
                rc6.reset();
            },
            ReceiverState::Error(_err) => rc6.reset(),
            _ => (),
        }
    }

    *PINVAL = new_pinval;
    *COUNT = COUNT.wrapping_add(1);
}