//! Hardware port pin wiring and indicator LEDs for the ATmega32U4 (SparkFun Pro Micro / Arduino Leonardo).
//!
//! Layout features:
//! - `layout-promicro-standard` (default): Matches `UnlicProController/UnlicProController.ino`
//!   - `A` = D3 (`PD0`), `B` = D1/TX (`PD3`) (or A1/`PF6` in classic), `X` = A0 (`PF7`), `Y` = D15/SCK (`PB1`)
//!   - `L` = A1 (`PF6`), `R` = D5 (`PC6`), `ZL` = A2 (`PF5`), `ZR` = D0/RX (`PD2`)
//!   - `Minus`/`Select` = D14/MISO (`PB3`), `Plus`/`Start` = D10 (`PB6`), `Home` = D16/MOSI (`PB2`), `Shift` = D2 (`PD1`)
//!   - `Up` = D7 (`PE6`), `Down` = D8 (`PB4`), `Left` = D6 (`PD7`), `Right` = D9 (`PB5`)
//!   - Mode LEDs: Red = A3 (`PF4`), Blue = D4 (`PD4`)
//! - `layout-promicro-classic`: Matches `origin/hid` button mask swap (`L`/`R`/`ZL`/`ZR` and `Minus`/`Plus`/`Home` permuted).

#[cfg(not(any(
    feature = "layout-promicro-standard",
    feature = "layout-promicro-classic"
)))]
compile_error!(
    "Select a Pro Micro layout feature: layout-promicro-standard or layout-promicro-classic"
);

#[cfg(all(
    feature = "layout-promicro-standard",
    feature = "layout-promicro-classic"
))]
compile_error!(
    "Pro Micro layout features are mutually exclusive; pass --no-default-features --features <layout>"
);

use arduino_hal::port::{
    mode::{Input, Output, PullUp},
    Pin,
};
use mumen_core::{ButtonMask, Debouncer, DirectionalMode, LogicalInputs};

const DEBOUNCE_SAMPLES: u8 = 4;

/// Physical GPIO inputs and mode indicator LEDs on the ATmega32U4.
pub struct ProMicroIo {
    switches: [Pin<Input<PullUp>>; 16],
    debouncers: [Debouncer; 16],
    led_red: Pin<Output>,
    led_blue: Pin<Output>,
}

impl ProMicroIo {
    /// Configure all 16 switch pins as internal pull-up inputs and the 2 mode LEDs as outputs.
    pub fn new(pins: arduino_hal::Pins) -> Self {
        let mut led_red = pins.a3.into_output().downgrade();
        let mut led_blue = pins.d4.into_output().downgrade();
        // Default mode is D-pad (Blue HIGH, Red LOW)
        led_red.set_low();
        led_blue.set_high();

        let switches = [
            pins.d3.into_pull_up_input().downgrade(),   // 0: A (PD0)
            pins.d1.into_pull_up_input().downgrade(),   // 1: B (PD3 / TX0)
            pins.a0.into_pull_up_input().downgrade(),   // 2: X (PF7 / A0)
            pins.sck.into_pull_up_input().downgrade(),  // 3: Y (PB1 / D15 / SCK)
            pins.a1.into_pull_up_input().downgrade(),   // 4: L / LB (PF6 / A1)
            pins.d5.into_pull_up_input().downgrade(),   // 5: R / RB (PC6 / D5)
            pins.a2.into_pull_up_input().downgrade(),   // 6: ZL / LT (PF5 / A2)
            pins.d0.into_pull_up_input().downgrade(),   // 7: ZR / RT (PD2 / RX1)
            pins.miso.into_pull_up_input().downgrade(), // 8: Select / Minus (PB3 / D14 / MISO)
            pins.d10.into_pull_up_input().downgrade(),  // 9: Start / Plus (PB6 / D10)
            pins.mosi.into_pull_up_input().downgrade(), // 10: Home (PB2 / D16 / MOSI)
            pins.d2.into_pull_up_input().downgrade(),   // 11: Shift (PD1 / D2)
            pins.d7.into_pull_up_input().downgrade(),   // 12: Up (PE6 / D7)
            pins.d8.into_pull_up_input().downgrade(),   // 13: Down (PB4 / D8)
            pins.d6.into_pull_up_input().downgrade(),   // 14: Left (PD7 / D6)
            pins.d9.into_pull_up_input().downgrade(),   // 15: Right (PB5 / D9)
        ];

        Self {
            switches,
            debouncers: [Debouncer::new(DEBOUNCE_SAMPLES); 16],
            led_red,
            led_blue,
        }
    }

    /// Sample and debounce all 16 switches, returning a [`LogicalInputs`] frame.
    pub fn sample(&mut self) -> LogicalInputs {
        let mut pressed = [false; 16];
        for i in 0..16 {
            pressed[i] = self.debouncers[i].update(self.switches[i].is_low());
        }

        let mut inputs = LogicalInputs::neutral();

        if pressed[0] {
            inputs.buttons |= ButtonMask::A;
        }
        if pressed[1] {
            inputs.buttons |= ButtonMask::B;
        }
        if pressed[2] {
            inputs.buttons |= ButtonMask::X;
        }
        if pressed[3] {
            inputs.buttons |= ButtonMask::Y;
        }

        if cfg!(feature = "layout-promicro-classic") && !cfg!(feature = "layout-promicro-standard")
        {
            // Classic `origin/hid` button permutation
            if pressed[4] {
                inputs.buttons |= ButtonMask::R;
            }
            if pressed[5] {
                inputs.buttons |= ButtonMask::ZR;
            }
            if pressed[6] {
                inputs.buttons |= ButtonMask::L;
            }
            if pressed[7] {
                inputs.buttons |= ButtonMask::ZL;
            }
            if pressed[8] {
                inputs.buttons |= ButtonMask::HOME;
            }
            if pressed[9] {
                inputs.buttons |= ButtonMask::MINUS;
            }
            if pressed[10] {
                inputs.buttons |= ButtonMask::PLUS;
            }
        } else {
            // Standard `UnlicProController` mapping
            if pressed[4] {
                inputs.buttons |= ButtonMask::L;
            }
            if pressed[5] {
                inputs.buttons |= ButtonMask::R;
            }
            if pressed[6] {
                inputs.buttons |= ButtonMask::ZL;
            }
            if pressed[7] {
                inputs.buttons |= ButtonMask::ZR;
            }
            if pressed[8] {
                inputs.buttons |= ButtonMask::MINUS;
            }
            if pressed[9] {
                inputs.buttons |= ButtonMask::PLUS;
            }
            if pressed[10] {
                inputs.buttons |= ButtonMask::HOME;
            }
        }

        inputs.shift = pressed[11];
        inputs.dpad.up = pressed[12];
        inputs.dpad.down = pressed[13];
        inputs.dpad.left = pressed[14];
        inputs.dpad.right = pressed[15];

        inputs
    }

    /// Update the Red (`A3`) and Blue (`D4`) mode indicator LEDs to reflect the active [`DirectionalMode`].
    #[inline]
    pub fn set_mode_leds(&mut self, mode: DirectionalMode) {
        match mode {
            DirectionalMode::Dpad => {
                self.led_red.set_low();
                self.led_blue.set_high();
            }
            DirectionalMode::Analog => {
                self.led_red.set_high();
                self.led_blue.set_low();
            }
            DirectionalMode::Smash => {
                self.led_red.set_high();
                self.led_blue.set_high();
            }
        }
    }
}
