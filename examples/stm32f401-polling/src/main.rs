#![no_main]
#![no_std]
#![allow(deprecated)]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nucleo_f401re::{
    gpio::{gpiob::PB8, Floating, Input},
    peripheral::Peripherals,
    prelude::*,
    stm32,
    stm32::interrupt,
    timer::{Event, Timer},
    Interrupt,
};
use panic_semihosting as _;
use infrared::{
    nec::{NecResult, NecCommand, NecReceiver},
    Receiver, State as ReceiverState,
    Remote,
    remotes::SamsungTv,
};

use heapless::consts::*;
use heapless::spsc::Queue;

// 50 us = 20_000 Hz
const FREQ: u32 = 20_000;

// Global data
static mut IRPIN: Option<PB8<Input<Floating>>> = None;
static mut NEC: Option<NecReceiver<SamsungTv>> = None;
static mut CQ: Option<Queue<NecResult<SamsungTv>, U8>> = None;


#[entry]
fn main() -> ! {
    let device = stm32::Peripherals::take().unwrap();
    let mut core = Peripherals::take().unwrap();

    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpiob = device.GPIOB.split();
    let irpin = gpiob.pb8.into_floating_input();

    unsafe {
        IRPIN.replace(irpin);
    }

    unsafe {
        NEC.replace(NecReceiver::new(FREQ));
    }

    // Setup the timer for 50us operation
    let mut timer2 = Timer::tim2(device.TIM2, FREQ.hz(), clocks);
    timer2.listen(Event::TimeOut);

    // Enable the external interrupt
    core.NVIC.enable(Interrupt::TIM2);

    unsafe { CQ = Some(Queue::new()) };
    let mut consumer = unsafe { CQ.as_mut().unwrap().split().1 };

    loop {
        let res = consumer.dequeue();

        if let Some(ReceiverState::Done(cmd)) = res {
            match cmd {
                NecCommand::Payload(c) => {
                    hprintln!("{:?}", c.action()).unwrap();
                }
                NecCommand::Repeat => hprintln!("repeat").unwrap(),
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
    unsafe { (*stm32::TIM2::ptr()).sr.modify(|_, w| w.uif().clear_bit()); }

    // Read the value of the pin (active low)
    let pinval = unsafe { IRPIN.as_ref().unwrap().is_low() };

    if *PINVAL != pinval {

        let nec = unsafe { NEC.as_mut().unwrap() };
        let state = nec.event(pinval, *COUNT);
        let mut producer = unsafe { CQ.as_mut().unwrap().split().0 };

        let is_err = state.is_err();
        let enqueue =  state.is_done() || state.is_err();

        if enqueue {
            producer.enqueue(state).ok().unwrap();
        }

        if is_err {
            nec.reset();
        }
    }

    *PINVAL = pinval;
    *COUNT += 1;
}


