#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use stm32f1xx_hal::{
    gpio::{
        gpiob::PB8,
        Floating, Input
    },
    pac,
    prelude::*,
    stm32::{interrupt, TIM2},
    timer::{Event, Timer, CountDownTimer},
};

#[allow(unused_imports)]
use infrared::{
    InfraredReceiver,
    nec::*,
    rc5::*,
    remotes::{
        rc5::*,
        nec::*,
        StandardButton,
    },
};

const TIMER_FREQ: u32 = 40_000;

static mut TIMER: Option<CountDownTimer<TIM2>> = None;

// Receiver
static mut RECEIVER: Option<InfraredReceiver<
    PB8<Input<Floating>>,
    Nec,
>> = None;


#[entry]
fn main() -> ! {
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

    let mut timer = Timer::tim2(device.TIM2, &clocks, &mut rcc.apb1)
        .start_count_down(TIMER_FREQ.hz());

    timer.listen(Event::Update);

    let receiver = InfraredReceiver::new(pin, TIMER_FREQ);

    // Safe because the devices are only used from in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        RECEIVER.replace(receiver);
    }

    unsafe {
        // Enable the timer interrupt
        pac::NVIC::unmask(pac::Interrupt::TIM2);
    }

    hprintln!("Ready!").unwrap();

    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    static mut SAMPLECOUNTER: u32 = 0;

    let receiver = unsafe { RECEIVER.as_mut().unwrap() };

    if let Ok(Some(button)) = receiver.sample_as_button::<SpecialForMp3>(*SAMPLECOUNTER) {
        use StandardButton::*;

        match button {
            Play_Paus => hprintln!("Play was pressed!").unwrap(),
            Power => hprintln!("Power on/off").unwrap(),
            _ => hprintln!("Button: {:?}", button).unwrap(),
        };
    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();

    *SAMPLECOUNTER = SAMPLECOUNTER.wrapping_add(1);
}

