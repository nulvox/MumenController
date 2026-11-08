//! Hardware GPIO & ADC pin wiring and compile-time button layout profiles for Teensy 4.0.
//!
//! Exactly one layout feature should be enabled at compile time:
//! - `layout-standard` (default): 20-pin Teensy 4.0 layout (`P0` Lock w/ pull-down, `P1`..`P19` buttons/D-pad/AnalogToggles w/ pull-up)
//! - `layout-alternate`: 14-pin Teensy 4.0 layout (`A`/`B` swapped on `P11`/`P14`, no `L2/R2/L3/R3/Lock/AnalogToggles`)
//! - `layout-d16`: 16-pin sequential layout (`P0`..`P15`) with `Capture` on `P11` (`switchspec/d16.yml`)
//! - `layout-d16s`: 16-pin sequential layout (`P0`..`P15`) with `Shift` modifier on `P11` (`switchspec/d16s.yml`)
//! - `layout-a4d17`: 17-pin digital (`P0`..`P16`) + 4-axis ADC analog sticks (`P17`..`P20` on `ADC1`, `switchspec/a4d17.yml`)

#[cfg(not(any(
    feature = "layout-standard",
    feature = "layout-alternate",
    feature = "layout-d16",
    feature = "layout-d16s",
    feature = "layout-a4d17"
)))]
compile_error!(
    "Select a Teensy 4.0 layout feature: layout-standard, layout-alternate, layout-d16, layout-d16s, or layout-a4d17"
);

use mumen_core::{ButtonMask, Debouncer, LogicalInputs};
use teensy4_bsp::hal::{gpio, iomuxc};
use teensy4_bsp::pins::t40::Pins;

/// Pad configuration for active-low momentary switches (switch between GPIO and GND):
/// internal 100k pull-up resistor.
const PULLUP_CONFIG: iomuxc::Config =
    iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pullup100k));

/// Number of consecutive 1 ms samples required to transition a switch state.
const DEBOUNCE_SAMPLES: u8 = 4;

/// Helper to read an active-low input pin (`!pin.is_set()`) through a [`Debouncer`].
#[inline]
fn read_active_low<P>(pin: &gpio::Input<P>, debouncer: &mut Debouncer) -> bool {
    debouncer.update(!pin.is_set())
}

// ============================================================================
// 1. Standard 20-pin Teensy 4.0 Layout (`layout-standard`)
// ============================================================================
#[cfg(feature = "layout-standard")]
mod imp {
    use super::*;
    use teensy4_bsp::hal::adc;
    use teensy4_bsp::pins::t40::*;

    const PULLDOWN_CONFIG: iomuxc::Config =
        iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pulldown100k));

    pub struct BoardInputs {
        pin_a: gpio::Input<P14>,
        pin_b: gpio::Input<P11>,
        pin_x: gpio::Input<P9>,
        pin_y: gpio::Input<P16>,
        pin_l1: gpio::Input<P15>,
        pin_r1: gpio::Input<P10>,
        pin_l2: gpio::Input<P12>,
        pin_r2: gpio::Input<P13>,
        pin_l3: gpio::Input<P3>,
        pin_r3: gpio::Input<P2>,
        pin_select: gpio::Input<P18>,
        pin_start: gpio::Input<P17>,
        pin_home: gpio::Input<P8>,
        pin_up: gpio::Input<P1>,
        pin_down: gpio::Input<P6>,
        pin_left: gpio::Input<P7>,
        pin_right: gpio::Input<P19>,
        pin_t_analog_left: gpio::Input<P4>,
        pin_t_analog_right: gpio::Input<P5>,
        pin_lock: gpio::Input<P0>,
        debouncers: [Debouncer; 20],
    }

    impl BoardInputs {
        pub fn new(
            mut pins: Pins,
            gpio1: &mut gpio::Port<1>,
            gpio2: &mut gpio::Port<2>,
            gpio4: &mut gpio::Port<4>,
            _adc1: adc::Adc<1>,
        ) -> Self {
            iomuxc::configure(&mut pins.p0, PULLDOWN_CONFIG);
            iomuxc::configure(&mut pins.p1, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p2, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p3, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p4, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p5, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p6, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p7, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p8, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p9, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p10, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p11, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p12, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p13, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p14, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p15, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p16, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p17, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p18, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p19, PULLUP_CONFIG);

            Self {
                pin_a: gpio1.input(pins.p14),
                pin_b: gpio2.input(pins.p11),
                pin_x: gpio2.input(pins.p9),
                pin_y: gpio1.input(pins.p16),
                pin_l1: gpio1.input(pins.p15),
                pin_r1: gpio2.input(pins.p10),
                pin_l2: gpio2.input(pins.p12),
                pin_r2: gpio2.input(pins.p13),
                pin_l3: gpio4.input(pins.p3),
                pin_r3: gpio4.input(pins.p2),
                pin_select: gpio1.input(pins.p18),
                pin_start: gpio1.input(pins.p17),
                pin_home: gpio2.input(pins.p8),
                pin_up: gpio1.input(pins.p1),
                pin_down: gpio2.input(pins.p6),
                pin_left: gpio2.input(pins.p7),
                pin_right: gpio1.input(pins.p19),
                pin_t_analog_left: gpio4.input(pins.p4),
                pin_t_analog_right: gpio4.input(pins.p5),
                pin_lock: gpio1.input(pins.p0),
                debouncers: [Debouncer::new(DEBOUNCE_SAMPLES); 20],
            }
        }

        pub fn sample(&mut self) -> LogicalInputs {
            let mut inputs = LogicalInputs::neutral();

            if read_active_low(&self.pin_a, &mut self.debouncers[0]) {
                inputs.buttons |= ButtonMask::A;
            }
            if read_active_low(&self.pin_b, &mut self.debouncers[1]) {
                inputs.buttons |= ButtonMask::B;
            }
            if read_active_low(&self.pin_x, &mut self.debouncers[2]) {
                inputs.buttons |= ButtonMask::X;
            }
            if read_active_low(&self.pin_y, &mut self.debouncers[3]) {
                inputs.buttons |= ButtonMask::Y;
            }
            if read_active_low(&self.pin_l1, &mut self.debouncers[4]) {
                inputs.buttons |= ButtonMask::L;
            }
            if read_active_low(&self.pin_r1, &mut self.debouncers[5]) {
                inputs.buttons |= ButtonMask::R;
            }
            if read_active_low(&self.pin_l2, &mut self.debouncers[6]) {
                inputs.buttons |= ButtonMask::ZL;
            }
            if read_active_low(&self.pin_r2, &mut self.debouncers[7]) {
                inputs.buttons |= ButtonMask::ZR;
            }
            if read_active_low(&self.pin_l3, &mut self.debouncers[8]) {
                inputs.buttons |= ButtonMask::L3;
            }
            if read_active_low(&self.pin_r3, &mut self.debouncers[9]) {
                inputs.buttons |= ButtonMask::R3;
            }
            if read_active_low(&self.pin_select, &mut self.debouncers[10]) {
                inputs.buttons |= ButtonMask::MINUS;
            }
            if read_active_low(&self.pin_start, &mut self.debouncers[11]) {
                inputs.buttons |= ButtonMask::PLUS;
            }
            if read_active_low(&self.pin_home, &mut self.debouncers[12]) {
                inputs.buttons |= ButtonMask::HOME;
            }

            inputs.dpad.up = read_active_low(&self.pin_up, &mut self.debouncers[13]);
            inputs.dpad.down = read_active_low(&self.pin_down, &mut self.debouncers[14]);
            inputs.dpad.left = read_active_low(&self.pin_left, &mut self.debouncers[15]);
            inputs.dpad.right = read_active_low(&self.pin_right, &mut self.debouncers[16]);

            inputs.toggle_analog_left =
                read_active_low(&self.pin_t_analog_left, &mut self.debouncers[17]);
            inputs.toggle_analog_right =
                read_active_low(&self.pin_t_analog_right, &mut self.debouncers[18]);

            // In `d98a54b`, menu buttons were only enabled when `pin_lock.is_high()`,
            // meaning `lock_active` (suppressing menu buttons) is true when `pin_lock` is LOW.
            inputs.lock_active = read_active_low(&self.pin_lock, &mut self.debouncers[19]);

            inputs
        }
    }
}

// ============================================================================
// 2. Alternate 14-pin Teensy 4.0 Layout (`layout-alternate`)
// ============================================================================
#[cfg(all(feature = "layout-alternate", not(feature = "layout-standard")))]
mod imp {
    use super::*;
    use teensy4_bsp::hal::adc;
    use teensy4_bsp::pins::t40::*;

    pub struct BoardInputs {
        // Note: A and B are swapped relative to layout-standard (A on P11, B on P14)
        pin_a: gpio::Input<P11>,
        pin_b: gpio::Input<P14>,
        pin_x: gpio::Input<P9>,
        pin_y: gpio::Input<P16>,
        pin_l1: gpio::Input<P15>,
        pin_r1: gpio::Input<P10>,
        pin_select: gpio::Input<P18>,
        pin_start: gpio::Input<P17>,
        pin_home: gpio::Input<P8>,
        pin_up: gpio::Input<P1>,
        pin_down: gpio::Input<P6>,
        pin_left: gpio::Input<P7>,
        pin_right: gpio::Input<P19>,
        debouncers: [Debouncer; 13],
    }

    impl BoardInputs {
        pub fn new(
            mut pins: Pins,
            gpio1: &mut gpio::Port<1>,
            gpio2: &mut gpio::Port<2>,
            _gpio4: &mut gpio::Port<4>,
            _adc1: adc::Adc<1>,
        ) -> Self {
            iomuxc::configure(&mut pins.p1, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p6, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p7, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p8, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p9, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p10, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p11, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p14, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p15, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p16, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p17, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p18, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p19, PULLUP_CONFIG);

            Self {
                pin_a: gpio2.input(pins.p11),
                pin_b: gpio1.input(pins.p14),
                pin_x: gpio2.input(pins.p9),
                pin_y: gpio1.input(pins.p16),
                pin_l1: gpio1.input(pins.p15),
                pin_r1: gpio2.input(pins.p10),
                pin_select: gpio1.input(pins.p18),
                pin_start: gpio1.input(pins.p17),
                pin_home: gpio2.input(pins.p8),
                pin_up: gpio1.input(pins.p1),
                pin_down: gpio2.input(pins.p6),
                pin_left: gpio2.input(pins.p7),
                pin_right: gpio1.input(pins.p19),
                debouncers: [Debouncer::new(DEBOUNCE_SAMPLES); 13],
            }
        }

        pub fn sample(&mut self) -> LogicalInputs {
            let mut inputs = LogicalInputs::neutral();

            if read_active_low(&self.pin_a, &mut self.debouncers[0]) {
                inputs.buttons |= ButtonMask::A;
            }
            if read_active_low(&self.pin_b, &mut self.debouncers[1]) {
                inputs.buttons |= ButtonMask::B;
            }
            if read_active_low(&self.pin_x, &mut self.debouncers[2]) {
                inputs.buttons |= ButtonMask::X;
            }
            if read_active_low(&self.pin_y, &mut self.debouncers[3]) {
                inputs.buttons |= ButtonMask::Y;
            }
            if read_active_low(&self.pin_l1, &mut self.debouncers[4]) {
                inputs.buttons |= ButtonMask::L;
            }
            if read_active_low(&self.pin_r1, &mut self.debouncers[5]) {
                inputs.buttons |= ButtonMask::R;
            }
            if read_active_low(&self.pin_select, &mut self.debouncers[6]) {
                inputs.buttons |= ButtonMask::MINUS;
            }
            if read_active_low(&self.pin_start, &mut self.debouncers[7]) {
                inputs.buttons |= ButtonMask::PLUS;
            }
            if read_active_low(&self.pin_home, &mut self.debouncers[8]) {
                inputs.buttons |= ButtonMask::HOME;
            }

            inputs.dpad.up = read_active_low(&self.pin_up, &mut self.debouncers[9]);
            inputs.dpad.down = read_active_low(&self.pin_down, &mut self.debouncers[10]);
            inputs.dpad.left = read_active_low(&self.pin_left, &mut self.debouncers[11]);
            inputs.dpad.right = read_active_low(&self.pin_right, &mut self.debouncers[12]);

            inputs
        }
    }
}

// ============================================================================
// 3. Sequential 16-pin Digital Layouts (`layout-d16` and `layout-d16s`)
//    Matches `switchspec/d16.yml` (P11 = Capture) and `switchspec/d16s.yml` (P11 = Shift)
// ============================================================================
#[cfg(all(
    any(feature = "layout-d16", feature = "layout-d16s"),
    not(feature = "layout-standard"),
    not(feature = "layout-alternate")
))]
mod imp {
    use super::*;
    use teensy4_bsp::hal::adc;
    use teensy4_bsp::pins::t40::*;

    pub struct BoardInputs {
        pin_a: gpio::Input<P0>,
        pin_b: gpio::Input<P1>,
        pin_x: gpio::Input<P2>,
        pin_y: gpio::Input<P3>,
        pin_l1: gpio::Input<P4>,
        pin_r1: gpio::Input<P5>,
        pin_l2: gpio::Input<P6>,
        pin_r2: gpio::Input<P7>,
        pin_select: gpio::Input<P8>,
        pin_start: gpio::Input<P9>,
        pin_home: gpio::Input<P10>,
        pin_aux: gpio::Input<P11>, // Capture (d16) or Shift (d16s)
        pin_up: gpio::Input<P12>,
        pin_down: gpio::Input<P13>,
        pin_left: gpio::Input<P14>,
        pin_right: gpio::Input<P15>,
        debouncers: [Debouncer; 16],
    }

    impl BoardInputs {
        pub fn new(
            mut pins: Pins,
            gpio1: &mut gpio::Port<1>,
            gpio2: &mut gpio::Port<2>,
            gpio4: &mut gpio::Port<4>,
            _adc1: adc::Adc<1>,
        ) -> Self {
            iomuxc::configure(&mut pins.p0, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p1, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p2, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p3, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p4, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p5, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p6, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p7, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p8, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p9, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p10, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p11, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p12, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p13, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p14, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p15, PULLUP_CONFIG);

            Self {
                pin_a: gpio1.input(pins.p0),
                pin_b: gpio1.input(pins.p1),
                pin_x: gpio4.input(pins.p2),
                pin_y: gpio4.input(pins.p3),
                pin_l1: gpio4.input(pins.p4),
                pin_r1: gpio4.input(pins.p5),
                pin_l2: gpio2.input(pins.p6),
                pin_r2: gpio2.input(pins.p7),
                pin_select: gpio2.input(pins.p8),
                pin_start: gpio2.input(pins.p9),
                pin_home: gpio2.input(pins.p10),
                pin_aux: gpio2.input(pins.p11),
                pin_up: gpio2.input(pins.p12),
                pin_down: gpio2.input(pins.p13),
                pin_left: gpio1.input(pins.p14),
                pin_right: gpio1.input(pins.p15),
                debouncers: [Debouncer::new(DEBOUNCE_SAMPLES); 16],
            }
        }

        pub fn sample(&mut self) -> LogicalInputs {
            let mut inputs = LogicalInputs::neutral();

            if read_active_low(&self.pin_a, &mut self.debouncers[0]) {
                inputs.buttons |= ButtonMask::A;
            }
            if read_active_low(&self.pin_b, &mut self.debouncers[1]) {
                inputs.buttons |= ButtonMask::B;
            }
            if read_active_low(&self.pin_x, &mut self.debouncers[2]) {
                inputs.buttons |= ButtonMask::X;
            }
            if read_active_low(&self.pin_y, &mut self.debouncers[3]) {
                inputs.buttons |= ButtonMask::Y;
            }
            if read_active_low(&self.pin_l1, &mut self.debouncers[4]) {
                inputs.buttons |= ButtonMask::L;
            }
            if read_active_low(&self.pin_r1, &mut self.debouncers[5]) {
                inputs.buttons |= ButtonMask::R;
            }
            if read_active_low(&self.pin_l2, &mut self.debouncers[6]) {
                inputs.buttons |= ButtonMask::ZL;
            }
            if read_active_low(&self.pin_r2, &mut self.debouncers[7]) {
                inputs.buttons |= ButtonMask::ZR;
            }
            if read_active_low(&self.pin_select, &mut self.debouncers[8]) {
                inputs.buttons |= ButtonMask::MINUS;
            }
            if read_active_low(&self.pin_start, &mut self.debouncers[9]) {
                inputs.buttons |= ButtonMask::PLUS;
            }
            if read_active_low(&self.pin_home, &mut self.debouncers[10]) {
                inputs.buttons |= ButtonMask::HOME;
            }

            let aux_pressed = read_active_low(&self.pin_aux, &mut self.debouncers[11]);
            if cfg!(feature = "layout-d16s") {
                inputs.shift = aux_pressed;
            } else if aux_pressed {
                inputs.buttons |= ButtonMask::CAPTURE;
            }

            inputs.dpad.up = read_active_low(&self.pin_up, &mut self.debouncers[12]);
            inputs.dpad.down = read_active_low(&self.pin_down, &mut self.debouncers[13]);
            inputs.dpad.left = read_active_low(&self.pin_left, &mut self.debouncers[14]);
            inputs.dpad.right = read_active_low(&self.pin_right, &mut self.debouncers[15]);

            inputs
        }
    }
}

// ============================================================================
// 4. 17-pin Digital + 4-axis Analog Layout (`layout-a4d17`)
//    Matches `switchspec/a4d17.yml` (P0..P16 digital + P17..P20 ADC1 analog sticks)
// ============================================================================
#[cfg(all(
    feature = "layout-a4d17",
    not(feature = "layout-standard"),
    not(feature = "layout-alternate"),
    not(feature = "layout-d16"),
    not(feature = "layout-d16s")
))]
mod imp {
    use super::*;
    use teensy4_bsp::hal::adc;
    use teensy4_bsp::pins::t40::*;

    pub struct BoardInputs {
        pin_a: gpio::Input<P0>,
        pin_b: gpio::Input<P1>,
        pin_x: gpio::Input<P2>,
        pin_y: gpio::Input<P3>,
        pin_l1: gpio::Input<P4>,
        pin_r1: gpio::Input<P5>,
        pin_l2: gpio::Input<P6>,
        pin_r2: gpio::Input<P7>,
        pin_select: gpio::Input<P8>,
        pin_start: gpio::Input<P9>,
        pin_home: gpio::Input<P10>,
        pin_l3: gpio::Input<P11>,
        pin_r3: gpio::Input<P12>,
        pin_up: gpio::Input<P13>,
        pin_down: gpio::Input<P14>,
        pin_left: gpio::Input<P15>,
        pin_right: gpio::Input<P16>,
        adc1: adc::Adc<1>,
        pin_lx: adc::AnalogInput<P17, 1>,
        pin_ly: adc::AnalogInput<P18, 1>,
        pin_rx: adc::AnalogInput<P19, 1>,
        pin_ry: adc::AnalogInput<P20, 1>,
        debouncers: [Debouncer; 17],
    }

    impl BoardInputs {
        pub fn new(
            mut pins: Pins,
            gpio1: &mut gpio::Port<1>,
            gpio2: &mut gpio::Port<2>,
            gpio4: &mut gpio::Port<4>,
            adc1: adc::Adc<1>,
        ) -> Self {
            iomuxc::configure(&mut pins.p0, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p1, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p2, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p3, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p4, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p5, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p6, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p7, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p8, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p9, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p10, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p11, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p12, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p13, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p14, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p15, PULLUP_CONFIG);
            iomuxc::configure(&mut pins.p16, PULLUP_CONFIG);

            Self {
                pin_a: gpio1.input(pins.p0),
                pin_b: gpio1.input(pins.p1),
                pin_x: gpio4.input(pins.p2),
                pin_y: gpio4.input(pins.p3),
                pin_l1: gpio4.input(pins.p4),
                pin_r1: gpio4.input(pins.p5),
                pin_l2: gpio2.input(pins.p6),
                pin_r2: gpio2.input(pins.p7),
                pin_select: gpio2.input(pins.p8),
                pin_start: gpio2.input(pins.p9),
                pin_home: gpio2.input(pins.p10),
                pin_l3: gpio2.input(pins.p11),
                pin_r3: gpio2.input(pins.p12),
                pin_up: gpio2.input(pins.p13),
                pin_down: gpio1.input(pins.p14),
                pin_left: gpio1.input(pins.p15),
                pin_right: gpio1.input(pins.p16),
                adc1,
                pin_lx: adc::AnalogInput::new(pins.p17),
                pin_ly: adc::AnalogInput::new(pins.p18),
                pin_rx: adc::AnalogInput::new(pins.p19),
                pin_ry: adc::AnalogInput::new(pins.p20),
                debouncers: [Debouncer::new(DEBOUNCE_SAMPLES); 17],
            }
        }

        #[inline]
        fn adc10_to_u8(sample: u16) -> u8 {
            // Default `imxrt-hal` ADC resolution is 10-bit (0..=1023); scale to 8-bit (0..=255)
            (sample.min(1023) >> 2) as u8
        }

        pub fn sample(&mut self) -> LogicalInputs {
            let mut inputs = LogicalInputs::neutral();

            if read_active_low(&self.pin_a, &mut self.debouncers[0]) {
                inputs.buttons |= ButtonMask::A;
            }
            if read_active_low(&self.pin_b, &mut self.debouncers[1]) {
                inputs.buttons |= ButtonMask::B;
            }
            if read_active_low(&self.pin_x, &mut self.debouncers[2]) {
                inputs.buttons |= ButtonMask::X;
            }
            if read_active_low(&self.pin_y, &mut self.debouncers[3]) {
                inputs.buttons |= ButtonMask::Y;
            }
            if read_active_low(&self.pin_l1, &mut self.debouncers[4]) {
                inputs.buttons |= ButtonMask::L;
            }
            if read_active_low(&self.pin_r1, &mut self.debouncers[5]) {
                inputs.buttons |= ButtonMask::R;
            }
            if read_active_low(&self.pin_l2, &mut self.debouncers[6]) {
                inputs.buttons |= ButtonMask::ZL;
            }
            if read_active_low(&self.pin_r2, &mut self.debouncers[7]) {
                inputs.buttons |= ButtonMask::ZR;
            }
            if read_active_low(&self.pin_select, &mut self.debouncers[8]) {
                inputs.buttons |= ButtonMask::MINUS;
            }
            if read_active_low(&self.pin_start, &mut self.debouncers[9]) {
                inputs.buttons |= ButtonMask::PLUS;
            }
            if read_active_low(&self.pin_home, &mut self.debouncers[10]) {
                inputs.buttons |= ButtonMask::HOME;
            }
            if read_active_low(&self.pin_l3, &mut self.debouncers[11]) {
                inputs.buttons |= ButtonMask::L3;
            }
            if read_active_low(&self.pin_r3, &mut self.debouncers[12]) {
                inputs.buttons |= ButtonMask::R3;
            }

            inputs.dpad.up = read_active_low(&self.pin_up, &mut self.debouncers[13]);
            inputs.dpad.down = read_active_low(&self.pin_down, &mut self.debouncers[14]);
            inputs.dpad.left = read_active_low(&self.pin_left, &mut self.debouncers[15]);
            inputs.dpad.right = read_active_low(&self.pin_right, &mut self.debouncers[16]);

            inputs.lx = Self::adc10_to_u8(self.adc1.read_blocking(&mut self.pin_lx));
            inputs.ly = Self::adc10_to_u8(self.adc1.read_blocking(&mut self.pin_ly));
            inputs.rx = Self::adc10_to_u8(self.adc1.read_blocking(&mut self.pin_rx));
            inputs.ry = Self::adc10_to_u8(self.adc1.read_blocking(&mut self.pin_ry));

            inputs
        }
    }
}

pub use imp::BoardInputs;
