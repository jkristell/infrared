#![no_std]
#![no_main]
#![allow(unused)]
#![allow(deprecated)]

use panic_semihosting as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
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
    nec::{Command as NecCmd, NecReceiver, NecResult},
    Receiver, State as ReceiverState,
    remotes::SpecialForMp3,
};

const FREQ: u32 = 20_000;

static mut TIMER: Option<Timer<TIM2>> = None;
static mut IRPIN: Option<PB8<Input<Floating>>> = None;
static mut CQ: Option<Queue<NecResult<SpecialForMp3>, U8>> = None;


static mut NEC: Option<NecReceiver<SpecialForMp3>> = None;

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

    unsafe {
        TIMER.replace(timer);
        IRPIN.replace(irpin);
        NEC.replace(NecReceiver::new(FREQ));
    }

    core.NVIC.enable(pac::Interrupt::TIM2);

    unsafe { CQ = Some(Queue::new()) };
    let mut consumer = unsafe { CQ.as_mut().unwrap().split().1 };
    let stim = &mut core.ITM.stim[0];

    loop {
        let res = consumer.dequeue();

        if let Some(ReceiverState::Done(cmd)) = res {
            match cmd {
                NecCmd::Payload(data) => {
                    hprintln!("cmd: {:?}", data).unwrap();
                }
                NecCmd::Repeat => hprintln!("repeat").unwrap(),
            }
        }
        else if let Some(ReceiverState::Err(e)) = res {
            hprintln!("Err: {:?}", e).unwrap();
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
        let rising = *PINVAL == false && new_pinval == true;

        let nec = unsafe { NEC.as_mut().unwrap() };
        let state = nec.event(rising, *COUNT);
        let mut producer = unsafe { CQ.as_mut().unwrap().split().0 };

        let is_err = state.is_err();
        let enqueue = state.is_done() || is_err;

        if enqueue {
            producer.enqueue(state).ok().unwrap();
        }

        if is_err {
            nec.reset();
        }
    }

    *PINVAL = new_pinval;
    *COUNT += 1;
}


