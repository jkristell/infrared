#![no_std]
#![no_main]
#![allow(unused)]
#![allow(deprecated)]

use panic_semihosting as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32f1xx_hal::{
    gpio::{gpiob::{PB8, PB9}, Floating, PushPull, Input, Alternate},
    pwm::{Pins, Pwm, C4},
    pac,
    prelude::*,
    stm32::{interrupt, TIM2, TIM4},
    timer::{Event, Timer},
};

use heapless::consts::*;
use heapless::spsc::Queue;

use infrared::{
    protocols::{NecCommand, NecVariant, NecReceiver, NecResult, NecError},
    Receiver, State as ReceiverState,
    Remote,
    remotes::SpecialForMp3,
};

const FREQ: u32 = 20_000;

static mut TIMER: Option<Timer<TIM2>> = None;
static mut IRPIN: Option<PB8<Input<Floating>>> = None;
static mut NEC: Option<NecReceiver<u32>> = None;

// Command Queue
static mut CQ: Option<Queue<NecCommand<u32>, U8>> = None;
// Error Queue
static mut EQ: Option<Queue<NecError, U8>> = None;


#[entry]
fn main() -> ! {
    let mut core = cortex_m::Peripherals::take().unwrap();
    let device = pac::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(16.mhz())
        .freeze(&mut flash.acr);

    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let mut irpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::tim2(device.TIM2, 20.khz(), clocks, &mut rcc.apb1);
    timer.listen(Event::Update);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        IRPIN.replace(irpin);
        NEC.replace(NecReceiver::new(NecVariant::Standard, FREQ));
    }

    core.NVIC.enable(pac::Interrupt::TIM2);

    // Initialize the queues
    unsafe { CQ = Some(Queue::new()) };
    unsafe { EQ = Some(Queue::new()) };

    let mut cmdq = unsafe { CQ.as_mut().unwrap().split().1 };
    let mut errq = unsafe { EQ.as_mut().unwrap().split().1 };

    // Main loop
    loop {
        if let Some(cmd) = cmdq.dequeue() {
            match cmd {
                NecCommand::Payload(cmd) => {
                    // Convert the u32 to a command for our remote
                    let cmd = SpecialForMp3::from(cmd);

                    if let Some(action) = cmd.action() {
                        hprintln!("cmd: {:?}", action).unwrap();
                    } else {
                        hprintln!("<unknown>").unwrap();
                    }
                }
                NecCommand::Repeat => hprintln!("REPEAT").unwrap(),
            }
        }

        if let Some(err) = errq.dequeue() {
            hprintln!("Err: {:?}", err).unwrap();
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

        let nec = unsafe { NEC.as_mut().unwrap() };
        let state = nec.event(rising, *COUNT);
        let mut cmdq = unsafe { CQ.as_mut().unwrap().split().0 };
        let mut errq = unsafe { EQ.as_mut().unwrap().split().0 };

        if let ReceiverState::Done(cmd) = state {
            cmdq.enqueue(cmd).ok().unwrap();
        } else if let ReceiverState::Err(e) = state {
            errq.enqueue(e).ok().unwrap();
            nec.reset();
        }
    }

    *PINVAL = new_pinval;
    *COUNT += 1;
}


