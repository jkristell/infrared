#![no_main]
#![no_std]

use bluepill_examples as _;

use cortex_m::asm;

use infrared::{
    protocol::NecApple,
    remotecontrol::{nec::Apple2009, Action, Button},
    Receiver,
};

use defmt::info;

use stm32f1xx_hal::timer::fugit::TimerInstantU32;
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Edge, ExtiPin, Floating, Input},
    pac,
    prelude::*,
    usb::{Peripheral, UsbBus, UsbBusType},
};

use usb_device::{bus, prelude::*};
use usbd_hid::{
    descriptor::{generator_prelude::*, MediaKey, MediaKeyboardReport},
    hid_class::HIDClass,
};

#[rtic::app(device = stm32f1xx_hal::stm32, peripherals = true, dispatchers = [USART1])]
mod app {
    use super::*;

    const TIM_FREQ: u32 = 100_000;

    /// The pin connected to the infrared receiver module
    type IrPin = PB8<Input<Floating>>;
    type IrProto = NecApple;
    type IrRemote = Apple2009;
    type IrReceiver = Receiver<IrProto, IrPin, TimerInstantU32<TIM_FREQ>, Button<IrRemote>>;

    #[monotonic(binds = TIM3, default = true)]
    type Monotonic = stm32f1xx_hal::timer::MonoTimer<pac::TIM3, TIM_FREQ>;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBusType>,
        usb_kbd: HIDClass<'static, UsbBusType>,
    }

    #[local]
    struct Local {
        receiver: IrReceiver,
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
        asm::delay(clocks.sysclk().raw() / 100);

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        let usb_bus = cx.local.usb_bus;
        usb_bus.replace(UsbBus::new(usb));

        let usb_kbd = HIDClass::new(usb_bus.as_ref().unwrap(), MediaKeyboardReport::desc(), 64);

        let usb_dev = UsbDeviceBuilder::new(usb_bus.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Infrared")
            .product("Mediakeyboard")
            .serial_number("TEST")
            .device_class(0x03) // HID
            .build();

        let mut gpiob = cx.device.GPIOB.split();
        let mut pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);
        pin.make_interrupt_source(&mut afio);
        pin.trigger_on_edge(&cx.device.EXTI, Edge::RisingFalling);
        pin.enable_interrupt(&cx.device.EXTI);

        let receiver = Receiver::with_pin(1_000_000, pin);
        let mono = cx.device.TIM3.monotonic(&clocks);

        (
            Shared { usb_dev, usb_kbd },
            Local { receiver },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        info!("Setup done. In idle");
        loop {
            continue;
        }
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 3, shared = [usb_dev, usb_kbd])]
    fn usb_rx0(cx: usb_rx0::Context) {
        let usb_dev = cx.shared.usb_dev;
        let usb_kbd = cx.shared.usb_kbd;

        (usb_dev, usb_kbd).lock(|usb_dev, usb_kbd| {
            super::usb_poll(usb_dev, usb_kbd);
        });
    }

    #[task(binds = EXTI9_5, local = [receiver])]
    fn ir_rx(cx: ir_rx::Context) {
        let now = monotonics::Monotonic::now();
        let receiver = cx.local.receiver;

        if let Ok(Some(button)) = receiver.event_instant(now) {
            if let Some(action) = button.action() {
                info!("{:?}", button);
                let key = super::action_to_mediakey(action);
                info!("{:?}", defmt::Debug2Format(&key));
                keydown::spawn(key).unwrap();
            }
        }

        receiver.pin_mut().clear_interrupt_pending_bit();
    }

    #[task(shared = [usb_kbd])]
    fn keydown(mut cx: keydown::Context, key: MediaKey) {
        cx.shared.usb_kbd.lock(|kbd| super::send_keycode(kbd, key));

        keyup::spawn_after(20.millis()).unwrap();
    }

    #[task(shared = [usb_kbd])]
    fn keyup(mut cx: keyup::Context) {
        cx.shared
            .usb_kbd
            .lock(|kbd| super::send_keycode(kbd, MediaKey::Zero));
    }
}

fn usb_poll<B: bus::UsbBus>(
    usb_dev: &mut UsbDevice<'static, B>,
    usb_kbd: &mut HIDClass<'static, B>,
) {
    usb_dev.poll(&mut [usb_kbd]);
}

fn send_keycode(kbd: &HIDClass<UsbBusType>, key: MediaKey) {
    let report = MediaKeyboardReport {
        usage_id: key.into(),
    };

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

fn action_to_mediakey(action: Action) -> MediaKey {
    match action {
        Action::Play_Pause => MediaKey::PlayPause,
        Action::Up => MediaKey::VolumeIncrement,
        Action::Down => MediaKey::VolumeDecrement,
        Action::Right => MediaKey::NextTrack,
        Action::Left => MediaKey::PrevTrack,
        Action::Stop => MediaKey::Stop,
        _ => MediaKey::Zero,
    }
}
