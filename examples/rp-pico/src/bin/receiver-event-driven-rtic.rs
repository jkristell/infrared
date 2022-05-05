#![no_std]
#![no_main]

// Run with `DEFMT_LOG=info cargo run --release --bin receiver-event-monotonic`

use rp_pico_examples as _;

#[rtic::app(device = rp_pico::pac, peripherals = true, dispatchers = [XIP_IRQ])]
mod app {
    use rp_pico::{
        hal::{
            clocks::init_clocks_and_plls,
            gpio::{bank0::Gpio12, Floating, Input, Interrupt, Pin},
            sio::Sio,
            watchdog::Watchdog,
        },
        XOSC_CRYSTAL_FREQ,
    };

    use rp2040_monotonic::Rp2040Monotonic;
    use rp_pico::hal;

    // IR receiver stuff
    use infrared::{protocol::*, Receiver};

    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type Monotonic = Rp2040Monotonic;

    // Useful aliases
    pub type IRPin = Pin<Gpio12, Input<Floating>>;
    pub type IrReceiver =
        Receiver<AppleNec, IRPin, <Monotonic as rtic_monotonic::Monotonic>::Instant>;

    #[shared]
    struct Shared {
        #[lock_free]
        ir_recv: IrReceiver,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut watchdog = Watchdog::new(c.device.WATCHDOG);
        let mut resets = c.device.RESETS;
        let _clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            c.device.XOSC,
            c.device.CLOCKS,
            c.device.PLL_SYS,
            c.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = Sio::new(c.device.SIO);
        // Set the pins to their default state
        let pins = hal::gpio::Pins::new(
            c.device.IO_BANK0,
            c.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        // Setup the IR stuff
        let ir_sensor_pin = pins.gpio12.into_floating_input();
        ir_sensor_pin.set_interrupt_enabled(Interrupt::EdgeLow, true);
        ir_sensor_pin.set_interrupt_enabled(Interrupt::EdgeHigh, true);

        let ir_recv = Receiver::with_fugit64(ir_sensor_pin);

        defmt::info!("Init done");

        let mono = Monotonic::new(c.device.TIMER);
        (Shared { ir_recv }, Local {}, init::Monotonics(mono))
    }

    // Our event-based interrupt handler for infrared stuff:
    #[task(
        binds = IO_IRQ_BANK0,
        priority = 1,
        shared = [ir_recv],
    )]
    fn do_ir(c: do_ir::Context) {
        let now = monotonics::Monotonic::now();

        let ir = c.shared.ir_recv;

        // Write out our debug info
        if let Ok(Some(cmd)) = ir.event_instant(now) {
            defmt::info!("Cmd: {:?}", cmd);
        }

        // Clear the interrupts
        let pin = ir.pin_mut();
        pin.clear_interrupt(Interrupt::EdgeLow);
        pin.clear_interrupt(Interrupt::EdgeHigh);
    }
}
