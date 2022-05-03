use debouncr::{debounce_8, Debouncer, Edge, Repeat4};
// use debouncer::debounce_8;
// use arduino_hal::port;
use arduino_hal;
use crate::report;
// use report::KeyData;

/// If the switch is a pull-up or pull-down type
pub enum SwitchType {
    PullUp,
    PullDown,
}

/// Process state information from a 2 state switch.
/// [Debouncr](https://github.com/dbrgn/debouncr/) with a 4 sample array is used for debouncing.
pub struct Switch<T> {
    pin: T,
    state: Debouncer<u8, Repeat4>,
    falling: bool,
    rising: bool,
    switch_type: SwitchType,
    double_threshold: Option<u32>,
    held_threshold: Option<u32>,
    was_pressed: bool,
    held_counter: u32,
    last_press_counter: u32,
    single_press: bool,
    double_press: bool,
}

// @TODO change the InputPin type to one that matches avr_hal
impl<T> Switch<T>
where
    T: InputPin,
    <T as InputPin>::Error: core::fmt::Debug,
{
    /// Create a new Switch.
    pub fn new(pin: T, switch_type: SwitchType) -> Self {
        Self {
            pin,
            state: debounce_8(true),
            falling: false,
            rising: false,
            switch_type,
            double_threshold: None,
            held_threshold: None,
            was_pressed: false,
            held_counter: 0,
            last_press_counter: 0,
            single_press: false,
            double_press: false,
        }
    }

    /// Set the threshold in number of calls to update.
    pub fn set_held_thresh(&mut self, held_threshold: Option<u32>) {
        self.held_threshold = if let Some(held_threshold) = held_threshold {
            Some(held_threshold)
        } else {
            None
        };
    }

    /// Set the threshold in number of calls to update.
    pub fn set_double_thresh(&mut self, double_threshold: Option<u32>) {
        self.double_threshold = if let Some(double_threshold) = double_threshold {
            Some(double_threshold)
        } else {
            None
        };
    }

    /// Read the state of the switch and update status. This should be called on a timer.
    pub fn update(&mut self) {
        let is_pressed = self.is_pressed();

        // Handle event
        if let Some(edge) = self.state.update(is_pressed) {
            match edge {
                Edge::Falling => self.falling = true,
                Edge::Rising => self.rising = true,
            }
        } else {
            self.falling = false;
            self.rising = false;
        }

        // Handle double press logic
        if let Some(double_threshold) = self.double_threshold {
            // If we exceed the threshold for a double press reset it
            // Otherwise the counter will eventually wrap around and panic
            if self.single_press {
                self.last_press_counter += 1;
                if self.last_press_counter > double_threshold {
                    self.single_press = false;
                }
            }

            if self.falling {
                if self.single_press && self.last_press_counter < double_threshold {
                    self.double_press = true;
                    self.single_press = false;
                } else {
                    self.single_press = true;
                    self.last_press_counter = 0;
                }
            } else {
                self.double_press = false;
            }
        }

        // Handle held counter
        if is_pressed {
            self.held_counter += 1;
        }
        if self.rising {
            self.held_counter = 0;
        }
    }

    /// If the switch state is high
    pub fn is_high(&self) -> bool {
        self.state.is_high()
    }

    /// If the switch state is low
    pub fn is_low(&self) -> bool {
        self.state.is_low()
    }

    /// If the switch is pressed
    pub fn is_pressed(&self) -> bool {
        match self.switch_type {
            SwitchType::PullUp => self.pin.is_low().unwrap(),
            SwitchType::PullDown => self.pin.is_high().unwrap(),
        }
    }

    /// If the switch is rising
    pub fn is_rising(&self) -> bool {
        self.rising
    }

    /// If the switch is falling
    pub fn is_falling(&self) -> bool {
        self.falling
    }

    /// If the switch is being held
    pub fn is_held(&self) -> bool {
        if let Some(held_threshold) = self.held_threshold {
            return self.falling && self.held_counter >= held_threshold;
        }
        false
    }

    /// If the switch pressed twice inside the provided threshold
    pub fn is_double(&self) -> bool {
        self.double_press
    }
}

// no nested structs, they say I'm a bad developer
pub struct GamePad {
    SwitchA: struct Switch,
    SwitchB: struct Switch,
    SwitchX: struct Switch,
    SwitchY: struct Switch,
    SwitchL1: struct Switch,
    SwitchR1: struct Switch,
    SwitchL2: struct Switch,
    SwitchR2: struct Switch,
    SwitchSelect: struct Switch,
    SwitchStart: struct Switch,
    SwitchHome: struct Switch,
    SwitchShift: struct Switch,
    SwitchUp: struct Switch,
    SwitchDown: struct Switch,
    SwitchLeft: struct Switch,
    SwitchRight: struct Switch,
}

// @TODO the remaining functions in this file should be a trait implemented for GamePad
// Write the constructor for the gamepad's switches
pub fn build_gamepad(pins: &[arduino_hal::port::Pin; 16]) -> GamePad {
    let mut switches = GamePad {
        SwitchA: Switch::new(pins[0].into_float(), SwitchType::PullUp),
        SwitchB: Switch::new(pins[1].into_float(), SwitchType::PullUp),
        SwitchX: Switch::new(pins[2].into_float(), SwitchType::PullUp),
        SwitchY: Switch::new(pins[3].into_float(), SwitchType::PullUp),
        SwitchL1: Switch::new(pins[4].into_float(), SwitchType::PullUp),
        SwitchR1: Switch::new(pins[5].into_float(), SwitchType::PullUp),
        SwitchL2: Switch::new(pins[6].into_float(), SwitchType::PullUp),
        SwitchR2: Switch::new(pins[7].into_float(), SwitchType::PullUp),
        SwitchSelect: Switch::new(pins[8].into_float(), SwitchType::PullUp),
        SwitchStart: Switch::new(pins[9].into_float(), SwitchType::PullUp),
        SwitchHome: Switch::new(pins[10].into_float(), SwitchType::PullUp),
        SwitchShift: Switch::new(pins[11].into_float(), SwitchType::PullUp),
        SwitchUp: Switch::new(pins[12].into_float(), SwitchType::PullUp),
        SwitchDown: Switch::new(pins[13].into_float(), SwitchType::PullUp),
        SwitchLeft: Switch::new(pins[14].into_float(), SwitchType::PullUp),
        SwitchRight: Switch::new(pins[15].into_float(), SwitchType::PullUp),
    };
    return switches;
}

// Poll the debouncers and update the gamepad's state
pub fn poll_debouncers(&mut gamepad_signals: GamePad) {
    for switch in gamepad_signals {
        switch.update();
    }
    return gamepad_signals;
}

// Read the state of the gamepad's switches into the report
pub fn read_gamepad_switches(&mut gamepad_signals: GamePad) -> report::KeyData {
    for switch in gamepad_signals {
        if switch.is_high() {
            gamepad_signals.report.buttons[switch.pin] = 1;
        }
    }
    return gamepad_signals;
}