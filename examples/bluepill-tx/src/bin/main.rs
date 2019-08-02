#![no_std]
#![no_main]
#![allow(unused)]
#![allow(deprecated)]

use panic_semihosting as _;

use cortex_m::asm::delay;
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
use stm32_usbd::UsbBus;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use heapless::consts::*;
use heapless::spsc::Queue;

use infrared::{
    protocols::{NecCommand,
                NecVariant, NecType,
                NecReceiver, NecResult, NecError,
                NecTransmitter,
    },
    Receiver, ReceiverState,
    Transmitter, TransmitterState,
    remote::Remote,
    protocols::nec::remotes::{SpecialForMp3, SpecialForMp3Action},
    protocols::nec::remotes::SamsungTv,
};
use infrared::remote::RemoteControl;

const FREQ: u32 = 20_000;

static mut TIMER: Option<Timer<TIM2>> = None;
static mut IRPIN: Option<PB8<Input<Floating>>> = None;
static mut NEC: Option<NecReceiver<u32>> = None;
static mut NECTX: Option<NecTransmitter> = None;
static mut PWM: Option<Pwm<TIM4, C4>> = None;

// Command Queue
static mut CQ: Option<Queue<NecCommand<u32>, U8>> = None;
// Error Queue
static mut EQ: Option<Queue<NecError, U8>> = None;

static mut TXQ: Option<Queue<u32, U8>> = None;



// Using PB4 and PB5 channels for TIM3 PWM output
struct MyChannels(PB9<Alternate<PushPull>>);
impl Pins<TIM4> for MyChannels {
    const REMAP: u8 = 0b00;
    const C1: bool = false;
    const C2: bool = false;
    const C3: bool = false;
    const C4: bool = true; // PB9
    type Channels = (Pwm<TIM4, C4>);
}



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

    // Setup USB

    let mut gpioa = device.GPIOA.split(&mut rcc.apb2);

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay(clocks.sysclk().0 / 100);

    let usb_dm = gpioa.pa11;
    let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

    let usb_bus = UsbBus::new(device.USB, (usb_dm, usb_dp));

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x5824, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();

    // End





    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let mut irpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::tim2(device.TIM2, 20.khz(), clocks, &mut rcc.apb1);
    timer.listen(Event::Update);

    // PWM
    let mut afio = device.AFIO.constrain(&mut rcc.apb2);
    let ir_tx = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

    let mut c4: Pwm<TIM4, C4> = device.TIM4.pwm(
        MyChannels(ir_tx),
        &mut afio.mapr,
        38.khz(),
        clocks,
        &mut rcc.apb1
    );
    // Set the duty cycle of channel 0 to 50%
    c4.set_duty(c4.get_max_duty() / 2);
    c4.disable();

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        IRPIN.replace(irpin);
        NEC.replace(NecReceiver::new(NecVariant::Standard, FREQ));

        let per: u32 = (1 * 1000) / (FREQ / 1000);
        NECTX.replace(NecTransmitter::new(NecType::Nec, per));

        PWM.replace(c4);
    }

    core.NVIC.enable(pac::Interrupt::TIM2);

    // Initialize the queues
    unsafe {
        CQ = Some(Queue::new());
        EQ = Some(Queue::new());
        TXQ = Some(Queue::new());
    };


    let remote = SpecialForMp3;
    hprintln!("{}", remote.encode(SpecialForMp3Action::Power)).unwrap();


    let mut cmdq = unsafe { CQ.as_mut().unwrap().split().1 };
    let mut errq = unsafe { EQ.as_mut().unwrap().split().1 };
    let mut txq = unsafe { TXQ.as_mut().unwrap().split().0 };

    loop {
        // Handle USB
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];

            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {

                    //TODO: Initiate TX
                    txq.enqueue(64u32).ok().unwrap();


                    // Echo back in upper case
                    for c in buf[0..count].iter_mut() {
                        if 0x61 <= *c && *c <= 0x7a {
                            *c = *c + 1;
                        }
                    }

                    let mut write_offset = 0;
                    while write_offset < count {
                        match serial.write(&buf[write_offset..count]) {
                            Ok(len) if len > 0 => {
                                write_offset += len;
                            },
                            _ => {},
                        }
                    }
                }
                _ => {}
            }
        }




        if let Some(cmd) = cmdq.dequeue() {
            match cmd {
                NecCommand::Payload(cmd) => {
                    // Convert the u32 to a command for our remote
                    let cmd = remote.decode(cmd);
                    hprintln!("cmd: {:?}", cmd).unwrap();
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

    let nectx = unsafe { NECTX.as_mut().unwrap() };

    let txstate = nectx.transmit(*COUNT);
    let mut txq = unsafe { TXQ.as_mut().unwrap().split().1 };
    let pwm = unsafe { PWM.as_mut().unwrap() };

    match txstate {
        TransmitterState::Idle => {
            pwm.disable();
            // Check queue
            if let Some(txcmd) = txq.dequeue() {
                let remote = SpecialForMp3;

                nectx.set_command(remote.encode(SpecialForMp3Action::Power));
            }
        },
        TransmitterState::Transmit(true) => pwm.enable(),
        TransmitterState::Transmit(false) => pwm.disable(),
        TransmitterState::Done => {
            pwm.disable();
        },
        TransmitterState::Err => {
            hprintln!("ERR").unwrap();
        },
    }


/*
    if *PINVAL != new_pinval {
        let rising = new_pinval;

        let nec = unsafe { NEC.as_mut().unwrap() };
        let state = nec.event(rising, *COUNT);
        let mut cmdq = unsafe { CQ.as_mut().unwrap().split().0 };
        let mut errq = unsafe { EQ.as_mut().unwrap().split().0 };

        if let ReceiverState::Done(cmd) = state {
            cmdq.enqueue(cmd).ok().unwrap();
        } else if let ReceiverState::Err(e) = state {
            //errq.enqueue(e).ok().unwrap();
            nec.reset();
        }
    }
*/
    *PINVAL = new_pinval;
    *COUNT += 1;
}


