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
    nec::remotes::*,
    nec::{NecCommand, NecError, NecReceiver, NecType},
    Receiver, ReceiverState, RemoteControl,
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
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let irpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::tim2(device.TIM2, 20.khz(), clocks, &mut rcc.apb1);
    timer.listen(Event::Update);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        IRPIN.replace(irpin);
        NEC.replace(NecReceiver::new(NecType::Standard, FREQ));
    }

    // Initialize the queues
    unsafe {
        CQ.replace(Queue::new());
        EQ.replace(Queue::new());
    };

    let mut cmdq = unsafe { CQ.as_mut().unwrap().split().1 };
    let mut errq = unsafe { EQ.as_mut().unwrap().split().1 };

    // Enable the timer interrupt
    core.NVIC.enable(pac::Interrupt::TIM2);

    // Main loop
    loop {
        if let Some(cmd) = cmdq.dequeue() {
            match cmd {
                NecCommand::Payload(cmd) => {
                    let remotecontrol = SpecialForMp3;
                    let cmd = remotecontrol.decode(cmd);

                    if let Some(action) = cmd {
                        hprintln!("cmd: {:?}", action).unwrap();
                    } else {
                        hprintln!("unknown command").unwrap();
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

        let mut cmdq = unsafe { CQ.as_mut().unwrap().split().0 };
        let mut errq = unsafe { EQ.as_mut().unwrap().split().0 };

        let nec = unsafe { NEC.as_mut().unwrap() };
        let state = nec.event(rising, *COUNT);

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
