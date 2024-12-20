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
mod usb;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use bsp::board;
    use bsp::{
        hal::{gpio, iomuxc},
        pins,
    };
    use teensy4_bsp::{self as bsp, hal::iomuxc::Pad};

    use imxrt_log as logging;

    // If you're using a Teensy 4.1 or MicroMod, you should eventually
    // change 't40' to 't41' or micromod, respectively.
    use board::t40 as my_board;

    use rtic_monotonics::systick::{Systick, *};

    use crate::usb::{KeyData, PadReport};
    type Input = gpio::Input<pins::t40::P7>;

    /// There are no resources shared across tasks.
    #[shared]
    struct Shared {
        keys: PadReport,
        // keydata: KeyData,
        pin_a: Input,
        pin_b: Input,
        pin_x: Input,
        pin_y: Input,
        pin_l1: Input,
        pin_r1: Input,
        pin_l2: Input,
        pin_r2: Input,
        pin_l3: Input,
        pin_r3: Input,
        pin_select: Input,
        pin_start: Input,
        pin_home: Input,
        pin_up: Input,
        pin_down: Input,
        pin_left: Input,
        pin_right: Input,
        pin_t_analog_left: Input,
        pin_t_analog_right: Input,
        pin_lock: Input,
        pin_rx: Input,
        pin_ry: Input,
        pin_lx: Input,
        pin_ly: Input,
    }

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
            // mut gpio3,
            mut gpio4,
            pins,
            usb,
            // adc1,
            // adc2,
            ..
        } = my_board(cx.device);

        // let led = board::led(&mut gpio2, pins.p13);
        let pin_a = gpio1.input(pins.p14);
        let pin_b = gpio2.input(pins.p11);
        let pin_x = gpio2.input(pins.p9);
        let pin_y = gpio1.input(pins.p16);
        let pin_l1 = gpio1.input(pins.p15);
        let pin_r1 = gpio2.input(pins.p10);
        let pin_l2 = gpio2.input(pins.p12);
        let pin_r2 = gpio2.input(pins.p13);
        let pin_l3 = gpio4.input(pins.p3);
        let pin_r3 = gpio4.input(pins.p2);
        let pin_select = gpio1.input(pins.p18);
        let pin_start = gpio1.input(pins.p17);
        let pin_home = gpio2.input(pins.p8);
        let pin_up = gpio1.input(pins.p1);
        let pin_down = gpio2.input(pins.p6);
        let pin_left = gpio2.input(pins.p7);
        let pin_right = gpio1.input(pins.p19);
        let pin_t_analog_left = gpio4.input(pins.p4);
        let pin_t_analog_right = gpio4.input(pins.p5);
        let pin_lock = gpio1.input(pins.p0);
        let pin_rx = gpio1.input(pins.p22);
        let pin_ry = gpio1.input(pins.p23);
        let pin_lx = gpio1.input(pins.p20);
        let pin_ly = gpio1.input(pins.p21);

        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();
        let keydata: KeyData = KeyData {
            buttons: 0,
            hat: 0,
            padding: 0,
            lx: 0,
            ly: 0,
            rx: 0,
            ry: 0,
        };
        let keys = PadReport::new(&keydata);
        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        blink::spawn().unwrap();
        (
            Shared {
                keys,
                pin_a,
                pin_b,
                pin_x,
                pin_y,
                pin_l1,
                pin_r1,
                pin_l2,
                pin_r2,
                pin_l3,
                pin_r3,
                pin_select,
                pin_start,
                pin_home,
                pin_up,
                pin_down,
                pin_left,
                pin_right,
                pin_t_analog_left,
                pin_t_analog_right,
                pin_lock,
                pin_rx,
                pin_ry,
                pin_lx,
                pin_ly,
            },
            Local { poller },
        )
    }

    #[task(shared = [ keys, pin_a, pin_b, pin_x, pin_y, pin_l1, pin_r1, pin_l2, pin_r2, pin_l3, pin_r3, pin_select, pin_start, pin_home, pin_up, pin_down, pin_left, pin_right, pin_t_analog_left, pin_t_analog_right, pin_lock, pin_rx, pin_ry, pin_lx, pin_ly  ])]
    async fn blink(cx: blink::Context) {
        loop {
            if cx.local.pin_t_analog_left.is_set() {
                cx.shared.keys.lx = 0;
            } else if cx.local.pin_t_analog_right.is_set() {
                cx.shared.keys.lx = 255;
            }
        }
    }

    #[task(binds = USB_OTG1, local = [ poller ])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
