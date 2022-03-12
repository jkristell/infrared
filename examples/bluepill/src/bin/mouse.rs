#![no_main]
#![no_std]

use bluepill_examples as _;

use defmt::{info, Debug2Format};
use defmt_rtt as _; // global logger
use panic_probe as _;

use infrared::{
    protocol::NecApple,
    receiver::{Event, PinInput},
    remotecontrol::{nec::Apple2009, Action, Button},
    Receiver,
};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Edge, ExtiPin, Floating, Input},
    pac,
    prelude::*,
    usb::{Peripheral, UsbBus, UsbBusType},
};

use stm32f1xx_hal::timer::fugit::TimerInstantU32;

use usb_device::{bus, prelude::*};
use usbd_hid::{
    descriptor::{generator_prelude::*, MouseReport},
    hid_class::HIDClass,
};

#[rtic::app(device = stm32f1xx_hal::stm32, peripherals = true, dispatchers = [USART1])]
mod app {
    use super::*;

    const MONOTIMER_FREQ: u32 = 100_000;

    /// The pin connected to the infrared receiver module
    type RxPin = PB8<Input<Floating>>;
    type IrProto = NecApple;
    type IrRemote = Apple2009;
    type IrReceiver = Receiver<IrProto, Event, PinInput<RxPin>, Button<IrRemote>>;

    #[monotonic(binds = TIM3, default = true)]
    type Monotonic = stm32f1xx_hal::timer::MonoTimer<pac::TIM3, MONOTIMER_FREQ>;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBusType>,
        usb_hid: HIDClass<'static, UsbBusType>,
    }

    #[local]
    struct Local {
        ir_rx: IrReceiver,
        last_edge_ts: TimerInstantU32<{ app::MONOTIMER_FREQ }>,
    }

    #[init(
        local = [usb_bus: Option<bus::UsbBusAllocator<UsbBusType>> = None]
    )]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();
        let mut afio = cx.device.AFIO.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(48.MHz())
            .pclk1(24.MHz())
            .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        let mut gpioa = cx.device.GPIOA.split();
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low();
        cortex_m::asm::delay(clocks.sysclk().raw() / 100);
        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);
        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        let usb_bus = cx.local.usb_bus;
        usb_bus.replace(UsbBus::new(usb));
        let usb_hid = HIDClass::new(usb_bus.as_ref().unwrap(), MouseReport::desc(), 60);

        let usb_dev = UsbDeviceBuilder::new(usb_bus.as_ref().unwrap(), UsbVidPid(0, 0x3821))
            .manufacturer("Infrared")
            .product("Mouse")
            .serial_number("InfrR12")
            .device_class(0xEF)
            .build();

        let rx_pin = {
            let mut gpiob = cx.device.GPIOB.split();
            let mut pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);
            pin.make_interrupt_source(&mut afio);
            pin.trigger_on_edge(&cx.device.EXTI, Edge::RisingFalling);
            pin.enable_interrupt(&cx.device.EXTI);
            pin
        };

        // Run the receiver with native resolution and let embedded time to the conversion
        let ir_rx = infrared::Receiver::with_pin(1_000_000, rx_pin);
        //NOTE: Or use the builder:
        // infrared::Receiver::builder()
        //      .nec_apple()
        //      .resolution(1_000_000)
        //      .remote_control(IrRemote::default())
        //      .pin(rx_pin)
        //      .build();

        let mono = cx.device.TIM3.monotonic(&clocks);

        let shared = Shared { usb_dev, usb_hid };

        let local = Local {
            ir_rx,
            last_edge_ts: Monotonic::zero(),
        };

        (shared, local, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        info!("Setup done: in idle");
        loop {
            continue;
        }
    }

    #[task(binds = EXTI9_5, priority = 2, local = [ir_rx, last_edge_ts])]
    fn ir_rx(cx: ir_rx::Context) {
        let last_event = cx.local.last_edge_ts;
        let ir_rx = cx.local.ir_rx;

        let now = monotonics::Monotonic::now();
        let dt = now
            .checked_duration_since(*last_event)
            .map(|v| v.to_micros());

        if let Some(us) = dt {
            if let Ok(Some(button)) = ir_rx.event(us) {
                let _ = process_ir_cmd::spawn(button).ok();
            }
        }

        ir_rx.pin().clear_interrupt_pending_bit();
        *last_event = now;
    }

    #[task(
        local = [repeated: u32 = 0],
    )]
    fn process_ir_cmd(cx: process_ir_cmd::Context, button: Button<Apple2009>) {
        let is_repeated = button.is_repeat();

        let repeats = cx.local.repeated;
        if !is_repeated {
            *repeats = 0;
        }
        *repeats += 1;

        info!("Received: {:?}, repeat: {}", Debug2Format(&button), *repeats);
        if let Some(action) = button.action() {
            let report = super::button_to_mousereport(action, *repeats);
            info!("{:?}", Debug2Format(&report));
            keydown::spawn(report).unwrap()
        }
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 3, shared = [usb_dev, usb_hid])]
    fn usb_rx0(cx: usb_rx0::Context) {
        let usb_dev = cx.shared.usb_dev;
        let usb_hid = cx.shared.usb_hid;

        (usb_dev, usb_hid).lock(|usb_dev, usb_hid| usb_dev.poll(&mut [usb_hid]));
    }

    #[task(shared = [usb_hid])]
    fn keydown(mut cx: keydown::Context, mr: MouseReport) {
        cx.shared
            .usb_hid
            .lock(|kbd| super::send_mousereport(kbd, mr));
    }
}

fn send_mousereport(kbd: &HIDClass<UsbBusType>, report: MouseReport) {
    loop {
        let r = kbd.push_input(&report);
        match r {
            Ok(_) => break,
            Err(UsbError::WouldBlock) => {
                continue;
            }
            Err(_) => break,
        }
    }
}

fn button_to_mousereport(action: Action, repeats: u32) -> MouseReport {
    // Add some basic acceleration
    let steps = match repeats {
        r @ 0..=6 => 1 << (r as i8),
        _ => 64,
    };

    let mut buttons = 0;
    let mut x = 0;
    let mut y = 0;

    match action {
        Action::Play_Pause => {
            // Hold the button long enough to get a repeat that we use to signal mouse button release
            buttons = u8::from(repeats == 0);
        }
        Action::Up => y = -steps,
        Action::Down => y = steps,
        Action::Right => x = steps,
        Action::Left => x = -steps,
        _ => (),
    };

    MouseReport {
        buttons,
        x,
        y,
        wheel: 0,
        pan: 0,
    }
}
