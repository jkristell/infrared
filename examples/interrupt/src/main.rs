#![no_main]
#![no_std]
#![allow(deprecated)]

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nucleo_f401re::{
    gpio::{gpiob::PB8, Floating, Input, Edge, ExtiPin},
    peripheral::Peripherals,
    prelude::*,
    stm32,
    stm32::interrupt,
    timer::{Event, Timer},
    Interrupt,
};
use panic_semihosting as _;
use infrared::{
    nec::{NecResult, NecCmd, NecReceiver},
    Receiver, State as ReceiverState,
};

use heapless::consts::*;
use heapless::spsc::Queue;

// 50 us = 20_000 Hz
const FREQ: u32 = 20_000;

// Global data
static mut IRPIN: Option<PB8<Input<Floating>>> = None;
static mut NEC: NecReceiver = NecReceiver::new(FREQ);
static mut CQ: Option<Queue<NecResult, U8>> = None;
static PCOUNTER: AtomicU32 = AtomicU32::new(0);


#[entry]
fn main() -> ! {
    let mut device = stm32::Peripherals::take().unwrap();
    let mut core = Peripherals::take().unwrap();

    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpiob = device.GPIOB.split();
    let mut irpin = gpiob.pb8.into_floating_input();

    irpin.make_interrupt_source(&mut device.SYSCFG);
    irpin.enable_interrupt(&mut device.EXTI);
    irpin.trigger_on_edge(&mut device.EXTI, Edge::RISING_FALLING);


    unsafe {
        IRPIN.replace(irpin);
    }

    // Setup the timer for 50us operation
    let mut timer2 = Timer::tim2(device.TIM2, FREQ.hz(), clocks);
    timer2.listen(Event::TimeOut);

    unsafe { CQ.replace(Queue::new()) };
    let mut consumer = unsafe { CQ.as_mut().unwrap().split().1 };

    // Enable the external interrupt
    core.NVIC.enable(Interrupt::TIM2);
    core.NVIC.enable(Interrupt::EXTI9_5);


    loop {
        let res = consumer.dequeue();

        // The hprints are done in interrupt free context. So they make us loose button presses

        if let Some(ReceiverState::Done(cmd)) = res {
            match cmd {
                NecCmd::Command(button) => {
                    if let Some(name) = command_to_str(button.command()) {
                        hprintln!("cmd: {}", name).unwrap();
                    } else {
                        hprintln!("unknown: {}", button.command()).unwrap();
                    }
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
fn EXTI9_5() {

    // Read the value of the pin (active low)
    let rising = unsafe { IRPIN.as_ref().unwrap().is_low() };
    let count = PCOUNTER.load(Ordering::Relaxed);

    // Clear the interrupt
    unsafe {
        (*stm32::EXTI::ptr()).pr.modify(|_, w| w.pr8().set_bit());
    }

    let nec = unsafe { &mut NEC };
    let state = nec.event(rising, count);
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

#[interrupt]
fn TIM2() {
    unsafe { (*stm32::TIM2::ptr()).sr.modify(|_, w| w.uif().clear_bit()); }
    PCOUNTER.fetch_add(1, Ordering::Relaxed);
}


// Mappings for "Special for MP3" Remote
fn command_to_str(cmd: u8) -> Option<&'static str> {
    match cmd {
        69 => Some("Power"),
        70 => Some("Mode"),
        71 => Some("Mute"),
        68 => Some("Play/Paus"),
        64 => Some("Prev"),
        67 => Some("Next"),
        7 => Some("Eq"),
        21 => Some("Minus"),
        9 => Some("Plus"),
        22 => Some("0"),
        25 => Some("Shuffle"),
        13 => Some("U/SD"),
        12 => Some("1"),
        24 => Some("2"),
        94 => Some("3"),
        8 => Some("4"),
        28 => Some("5"),
        90 => Some("6"),
        66 => Some("7"),
        82 => Some("8"),
        74 => Some("9"),
        _ => None,
    }
}

