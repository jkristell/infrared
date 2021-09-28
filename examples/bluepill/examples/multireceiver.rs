#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input, Edge, ExtiPin},
    pac,
    prelude::*,
    stm32::interrupt,
    time::{MonoTimer, Instant},
};

use infrared::{
    protocol::*,
    receiver::{MultiReceiver, PinInput},
};

type IrPin = PB8<Input<Floating>>;
//type IrReceiver = MultiReceiver<(Rc6, Nec, NecSamsung, Rc5, NecApple), PinInput<IrPin>, 5>;
type IrReceiver = MultiReceiver<(NecSamsung, Rc5, Rc6, NecApple, Nec, Denon), PinInput<IrPin>, 6>;

static mut RECEIVER: Option<IrReceiver> = None;
static mut MONO: Option<MonoTimer> = None;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let cp = cortex_m::Peripherals::take().unwrap();
    let d = pac::Peripherals::take().unwrap();

    let mut flash = d.FLASH.constrain();
    let mut rcc = d.RCC.constrain();
    let mut afio = d.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    let mut gpiob = d.GPIOB.split(&mut rcc.apb2);
    let mut inpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    inpin.make_interrupt_source(&mut afio);
    inpin.trigger_on_edge(&d.EXTI, Edge::RISING_FALLING);
    inpin.enable_interrupt(&d.EXTI);


    let mono = MonoTimer::new(cp.DWT, cp.DCB, clocks);
    let mono_freq = mono.frequency();


    let receiver = MultiReceiver::new(mono_freq.0, PinInput(inpin));

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        RECEIVER.replace(receiver);
        MONO.replace(mono);
    }

    // Enable the timer interrupt
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
    static mut LAST: Option<Instant> = None;

    let receiver = unsafe { RECEIVER.as_mut().unwrap() };
    let mono = unsafe { MONO.as_ref().unwrap() };

    if let Some(dt) = LAST.map(|i| i.elapsed()) {
        if let Ok(cmds) = receiver.event_iter(dt) {

            for cmd in cmds {
                rprintln!("cmd: {:?}", cmd);
            }
        }
    }

    LAST.replace(mono.now());

    receiver.pin().clear_interrupt_pending_bit();
}
