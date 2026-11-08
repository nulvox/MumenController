//! Logical controller inputs, directional modes (D-pad / Analog / Smash),
//! menu lock handling, and report generation.

use crate::hid::{
    ButtonMask, Hat, SwitchReport, STICK_CENTER, STICK_HALF_HIGH, STICK_HALF_LOW, STICK_MAX,
    STICK_MIN,
};
use crate::socd::{DpadRaw, SocdState};

/// Directional output mode for the lever / directional buttons.
///
/// Supports both:
/// - **Hardware toggle switches** (Teensy 4.0 `analog_left` / `analog_right` toggle inputs)
/// - **Software mode cycling** (Pro Micro `HOME + SHIFT` cycling `Dpad -> Analog -> Smash`)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DirectionalMode {
    /// Output directional buttons on the HAT switch (D-pad).
    #[default]
    Dpad,
    /// Output directional buttons on the Left Stick (or Right Stick when `shift` is held).
    Analog,
    /// Output directional buttons on the Left Stick (`0`/`255`), or half-tilt (`64`/`192`) when `shift` is held.
    Smash,
}

impl DirectionalMode {
    /// Advance to the next mode in the `Dpad -> Analog -> Smash -> Dpad` cycle.
    #[inline]
    pub const fn next(self) -> Self {
        match self {
            Self::Dpad => Self::Analog,
            Self::Analog => Self::Smash,
            Self::Smash => Self::Dpad,
        }
    }
}

/// Logical snapshot of all physical controller switches and axes for a single scan tick.
///
/// All boolean fields represent logical **active/pressed** states (`true` = button pressed
/// or toggle engaged), independent of whether the underlying hardware pin is active-low or active-high.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalInputs {
    /// Bitmask of standard controller buttons (`ButtonMask::A`, `ButtonMask::B`, etc.).
    pub buttons: u16,
    /// Raw directional switch states (Up, Down, Left, Right).
    pub dpad: DpadRaw,
    /// Hardware toggle switch routing directional inputs to the Left Stick (Teensy 4.0 layout).
    pub toggle_analog_left: bool,
    /// Hardware toggle switch routing directional inputs to the Right Stick (Teensy 4.0 layout).
    pub toggle_analog_right: bool,
    /// Tournament lock switch: when `true`, suppresses `PLUS` (`Start`), `MINUS` (`Select`), `HOME`, and `CAPTURE`.
    pub lock_active: bool,
    /// Shift modifier button (Pro Micro / `d16s` layout): modifies `Dpad`/`Analog`/`Smash` behavior and cycles modes with `HOME`.
    pub shift: bool,
    /// Base analog stick values (`128` center by default; overridden if physical ADC sticks or digital-to-analog modes are active).
    pub lx: u8,
    pub ly: u8,
    pub rx: u8,
    pub ry: u8,
}

impl Default for LogicalInputs {
    fn default() -> Self {
        Self::neutral()
    }
}

impl LogicalInputs {
    /// Neutral input state (no buttons pressed, sticks centered at `128`).
    #[inline]
    pub const fn neutral() -> Self {
        Self {
            buttons: ButtonMask::NONE,
            dpad: DpadRaw {
                up: false,
                down: false,
                left: false,
                right: false,
            },
            toggle_analog_left: false,
            toggle_analog_right: false,
            lock_active: false,
            shift: false,
            lx: STICK_CENTER,
            ly: STICK_CENTER,
            rx: STICK_CENTER,
            ry: STICK_CENTER,
        }
    }
}

/// Stateful controller processor combining SOCD cleaning, mode switching, lock masking,
/// and 8-byte [`SwitchReport`] construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerEngine {
    pub socd: SocdState,
    pub mode: DirectionalMode,
    mode_chord_latched: bool,
}

impl Default for ControllerEngine {
    fn default() -> Self {
        Self::new(SocdState::default())
    }
}

impl ControllerEngine {
    #[inline]
    pub const fn new(socd: SocdState) -> Self {
        Self {
            socd,
            mode: DirectionalMode::Dpad,
            mode_chord_latched: false,
        }
    }

    /// Process a single frame of [`LogicalInputs`] and produce the 8-byte [`SwitchReport`].
    pub fn update(&mut self, inputs: &LogicalInputs) -> SwitchReport {
        let mut report = SwitchReport {
            buttons: inputs.buttons,
            hat: Hat::Neutral,
            lx: inputs.lx,
            ly: inputs.ly,
            rx: inputs.rx,
            ry: inputs.ry,
            vendor: 0,
        };

        // 1. Check for software mode-cycle chord (SHIFT + HOME rising edge)
        let home_pressed = (inputs.buttons & ButtonMask::HOME) != 0;
        if inputs.shift && home_pressed {
            if !self.mode_chord_latched {
                self.mode = self.mode.next();
                self.mode_chord_latched = true;
            }
        } else {
            self.mode_chord_latched = false;
        }

        // 2. Apply tournament lock switch to suppress menu buttons
        if inputs.lock_active {
            report.buttons &= !ButtonMask::LOCKABLE_MENU_MASK;
        }

        // 3. Process directional inputs according to hardware toggles or software mode
        if inputs.toggle_analog_left || inputs.toggle_analog_right {
            if inputs.toggle_analog_left {
                let (x, y) = Self::digital_to_stick_axis(inputs.dpad, STICK_MIN, STICK_MAX);
                report.lx = x;
                report.ly = y;
            }
            if inputs.toggle_analog_right {
                let (x, y) = Self::digital_to_stick_axis(inputs.dpad, STICK_MIN, STICK_MAX);
                report.rx = x;
                report.ry = y;
            }
        } else {
            let cleaned = self.socd.resolve(inputs.dpad);
            match self.mode {
                DirectionalMode::Dpad => {
                    report.hat =
                        Hat::from_directions(cleaned.up, cleaned.down, cleaned.left, cleaned.right);
                }
                DirectionalMode::Analog => {
                    let (x, y) = Self::cleaned_to_stick_axis(cleaned, STICK_MIN, STICK_MAX);
                    if inputs.shift {
                        report.rx = x;
                        report.ry = y;
                    } else {
                        report.lx = x;
                        report.ly = y;
                    }
                }
                DirectionalMode::Smash => {
                    let (low, high) = if inputs.shift {
                        (STICK_HALF_LOW, STICK_HALF_HIGH)
                    } else {
                        (STICK_MIN, STICK_MAX)
                    };
                    let (x, y) = Self::cleaned_to_stick_axis(cleaned, low, high);
                    report.lx = x;
                    report.ly = y;
                }
            }
        }

        report
    }

    /// Convert SOCD-cleaned directions to `(x, y)` analog bytes using `(low, high)` bounds.
    ///
    /// Note: USB HID / Switch Y-axis uses `0` (`low`) for Up and `255` (`high`) for Down.
    #[inline]
    const fn cleaned_to_stick_axis(cleaned: DpadRaw, low: u8, high: u8) -> (u8, u8) {
        let x = if cleaned.left && !cleaned.right {
            low
        } else if cleaned.right && !cleaned.left {
            high
        } else {
            STICK_CENTER
        };

        let y = if cleaned.up && !cleaned.down {
            low
        } else if cleaned.down && !cleaned.up {
            high
        } else {
            STICK_CENTER
        };

        (x, y)
    }

    /// Convert raw directions to `(x, y)` analog bytes with SOCD fallback for hardware toggle switches.
    #[inline]
    const fn digital_to_stick_axis(dpad: DpadRaw, low: u8, high: u8) -> (u8, u8) {
        // Horizontal: Left + Right -> Neutral (128)
        let x = match (dpad.left, dpad.right) {
            (true, false) => low,
            (false, true) => high,
            _ => STICK_CENTER,
        };
        // Vertical: Up + Down -> Up (low = 0)
        let y = match (dpad.up, dpad.down) {
            (true, _) => low,
            (false, true) => high,
            (false, false) => STICK_CENTER,
        };
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpad_mode_updates_hat_even_without_toggles() {
        let mut engine = ControllerEngine::default();
        let mut inputs = LogicalInputs::neutral();
        inputs.buttons = ButtonMask::A | ButtonMask::B;
        inputs.dpad.up = true;
        inputs.dpad.right = true;

        let report = engine.update(&inputs);
        assert_eq!(report.buttons, ButtonMask::A | ButtonMask::B);
        assert_eq!(report.hat, Hat::UpRight);
        assert_eq!(report.lx, STICK_CENTER);
        assert_eq!(report.ly, STICK_CENTER);
    }

    #[test]
    fn regression_analog_toggle_still_emits_face_buttons_and_resets_when_released() {
        // Regression test for the bug in d98a54b where `*keys = updated_keys` was only
        // executed inside the `else` (non-analog-toggle) branch!
        let mut engine = ControllerEngine::default();
        let mut inputs = LogicalInputs::neutral();
        inputs.toggle_analog_left = true;
        inputs.buttons = ButtonMask::X;
        inputs.dpad.left = true;
        inputs.dpad.up = true;

        let report = engine.update(&inputs);
        assert_eq!(report.buttons, ButtonMask::X);
        assert_eq!(report.hat, Hat::Neutral);
        assert_eq!(report.lx, STICK_MIN);
        assert_eq!(report.ly, STICK_MIN);

        // Release inputs on next frame
        let inputs_released = LogicalInputs {
            toggle_analog_left: true,
            ..LogicalInputs::neutral()
        };
        let report2 = engine.update(&inputs_released);
        assert_eq!(report2.buttons, ButtonMask::NONE);
        assert_eq!(report2.lx, STICK_CENTER);
        assert_eq!(report2.ly, STICK_CENTER);
    }

    #[test]
    fn lock_switch_suppresses_only_menu_buttons() {
        let mut engine = ControllerEngine::default();
        let mut inputs = LogicalInputs::neutral();
        inputs.buttons = ButtonMask::A
            | ButtonMask::PLUS
            | ButtonMask::MINUS
            | ButtonMask::HOME
            | ButtonMask::CAPTURE;
        inputs.lock_active = true;

        let report = engine.update(&inputs);
        assert_eq!(report.buttons, ButtonMask::A);

        inputs.lock_active = false;
        let report_unlocked = engine.update(&inputs);
        assert_eq!(report_unlocked.buttons, inputs.buttons);
    }

    #[test]
    fn shift_plus_home_cycles_modes_once_per_press_and_smash_uses_half_tilt() {
        let mut engine = ControllerEngine::default();
        assert_eq!(engine.mode, DirectionalMode::Dpad);

        let mut chord = LogicalInputs::neutral();
        chord.shift = true;
        chord.buttons = ButtonMask::HOME;

        // First frame of holding Shift+Home -> transitions to Analog
        let _ = engine.update(&chord);
        assert_eq!(engine.mode, DirectionalMode::Analog);

        // Holding Shift+Home on subsequent frame does NOT double-cycle
        let _ = engine.update(&chord);
        assert_eq!(engine.mode, DirectionalMode::Analog);

        // Release chord, then press again -> transitions to Smash
        let _ = engine.update(&LogicalInputs::neutral());
        let _ = engine.update(&chord);
        assert_eq!(engine.mode, DirectionalMode::Smash);

        // In Smash mode: full tilt without Shift, half tilt (64/192) with Shift
        let mut smash_in = LogicalInputs::neutral();
        smash_in.dpad.up = true;
        smash_in.dpad.right = true;
        let full = engine.update(&smash_in);
        assert_eq!((full.lx, full.ly), (STICK_MAX, STICK_MIN));

        smash_in.shift = true;
        let half = engine.update(&smash_in);
        assert_eq!((half.lx, half.ly), (STICK_HALF_HIGH, STICK_HALF_LOW));
    }
}
