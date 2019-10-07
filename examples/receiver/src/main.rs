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

use infrared::{
    Receiver, ReceiverState,
    nec::*,
    rc5::*,
    rc6::*,
};

const FREQ: u32 = 40_000;

static mut TIMER: Option<Timer<TIM2>> = None;
static mut IRPIN: Option<PB8<Input<Floating>>> = None;

// Receivers
static mut NEC: Option<NecReceiver> = None;
static mut NES: Option<NecSamsungReceiver> = None;
static mut RC5: Option<Rc5Receiver> = None;
static mut RC6: Option<Rc6Receiver> = None;

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

    // Enable the timer interrupt
    core.NVIC.enable(pac::Interrupt::TIM2);

    hprintln!("Ready!").unwrap();

    loop {
        continue;
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
        let nes = unsafe { NES.as_mut().unwrap() };
        let rc5 = unsafe { RC5.as_mut().unwrap() };
        let rc6 = unsafe { RC6.as_mut().unwrap() };


        if let Some(cmd) = sample(nec, rising, *COUNT) {
            hprintln!("{:?}", cmd).unwrap();
            nec.reset();
        }

        if let Some(cmd) = sample(nes, rising, *COUNT) {
            hprintln!("{:?}", cmd).unwrap();
            nes.reset();
        }

        if let Some(cmd) = sample(rc5, rising, *COUNT) {
            use infrared_remotes::rc5::CdPlayer;
            use infrared_remotes::RemoteControl;

            // Print the command if recognized as a Rc5 CD-player command
            if let Some(decoded) = CdPlayer.decode(cmd) {
                hprintln!("{:?}", decoded).unwrap();
            } else {
                hprintln!("{:?}", cmd).unwrap();
            }

            rc5.reset();
        }

        if let Some(cmd) = sample(rc6, rising, *COUNT) {
            hprintln!("{:?}", cmd).unwrap();
            rc6.reset();
        }
    }

    *PINVAL = new_pinval;
    *COUNT = COUNT.wrapping_add(1);
}


fn sample<RECEIVER, CMD, ERR>(recv: &mut RECEIVER, edge: bool, t: u32) -> Option<CMD>
where
    RECEIVER: Receiver<Cmd=CMD, Err=ERR> {

    match recv.sample_edge(edge, t) {
        ReceiverState::Done(c) => {
            return Some(c);
        }
        ReceiverState::Error(_err) => {
            recv.reset();
        }
        _ => {}
    }

    None
}

