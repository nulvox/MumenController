//! The starter code slowly blinks the LED and sets up
//! USB logging. It periodically logs messages over USB.
//!
//! Despite targeting the Teensy 4.0, this starter code
//! should also work on the Teensy 4.1 and Teensy MicroMod.
//! You should eventually target your board! See inline notes.
//!
//! This template uses [RTIC v2](https://rtic.rs/2/book/en/)
//! for structuring the application.

#![no_std]
#![no_main]
#![feature(generic_arg_infer)]

use teensy4_panic as _;
// mod usb;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use bsp::board;
    use teensy4_bsp as bsp;

    use imxrt_log as logging;

    // If you're using a Teensy 4.1 or MicroMod, you should eventually
    // change 't40' to 't41' or micromod, respectively.
    use board::t40 as my_board;

    use rtic_monotonics::systick::{Systick, *};

    /// There are no resources shared across tasks.
    #[shared]
    struct Shared {}

    /// These resources are local to individual tasks.
    #[local]
    struct Local {
        /// The LED on pin 13.
        // led: board::Led,
        /// A poller to control USB logging.
        poller: logging::Poller,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut gpio1,
            mut gpio2,
            // gpio3,
            mut gpio4,
            pins,
            usb,
            // adc1,
            ..
        } = my_board(cx.device);

        // let led = board::led(&mut gpio2, pins.p13);
        let _pin_a = gpio1.input(pins.p14);
        let _pin_b = gpio1.input(pins.p23);
        let _pin_x = gpio2.input(pins.p9);
        let _pin_y = gpio1.input(pins.p16);
        let _pin_l1 = gpio1.input(pins.p15);
        let _pin_r1 = gpio2.input(pins.p10);
        let _pin_l2 = gpio2.input(pins.p12);
        let _pin_r2 = gpio2.input(pins.p13);
        let _pin_l3 = gpio4.input(pins.p3);
        let _pin_r3 = gpio4.input(pins.p2);
        let _pin_select = gpio1.input(pins.p18);
        let _pin_start = gpio1.input(pins.p17);
        let _pin_home = gpio2.input(pins.p8);
        let _pin_up = gpio1.input(pins.p1);
        let _pin_down = gpio2.input(pins.p6);
        let _pin_left = gpio2.input(pins.p7);
        let _pin_right = gpio1.input(pins.p19);
        let _pin_t_analog_left = gpio4.input(pins.p4);
        let _pin_t_analog_right = gpio4.input(pins.p5);
        let _pin_lock = gpio1.input(pins.p0);
        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        blink::spawn().unwrap();
        (Shared {}, Local { poller })
    }

    #[task()]
    async fn blink(_cx: blink::Context) {
        let mut count = 0u32;
        loop {
            // cx.local.led.toggle();
            Systick::delay(500.millis()).await;

            log::info!("Hello from your Teensy 4! The count is {count}");
            if count % 7 == 0 {
                log::warn!("Here's a warning at count {count}");
            }
            if count % 23 == 0 {
                log::error!("Here's an error at count {count}");
            }

            count = count.wrapping_add(1);
        }
    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
