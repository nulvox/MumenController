use debouncr::{debounce_8, Debouncer, Edge, Repeat8};
use arduino_hal;

// Define the array offsets for each switch
pub static SWITCH_A: usize = 0;
pub static SWITCH_B: usize = 1;
pub static SWITCH_X: usize = 2;
pub static SWITCH_Y: usize = 3;
pub static SWITCH_L1: usize = 4;
pub static SWITCH_R1: usize = 5;
pub static SWITCH_L2: usize = 6;
pub static SWITCH_R2: usize = 7;
pub static SWITCH_SELECT: usize = 8;
pub static SWITCH_START: usize = 9;
pub static SWITCH_HOME: usize = 10;
pub static SWITCH_SHIFT: usize = 11;
pub static SWITCH_UP: usize = 12;
pub static SWITCH_DOWN: usize = 13;
pub static SWITCH_LEFT: usize = 14;
pub static SWITCH_RIGHT: usize = 15;

/// If the switch is a pull-up or pull-down type
#[derive(Debug, Copy, Clone)]
pub enum SwitchType {
    PullUp,
    PullDown,
}

pub enum ButtonName {
    ButtonA,
    ButtonB,
    ButtonX,
    ButtonY,
    ButtonL1,
    ButtonR1,
    ButtonL2,
    ButtonR2,
    ButtonSelect,
    ButtonStart,
    ButtonHome,
    ButtonShift,
    ButtonUp,
    ButtonDown,
    ButtonLeft,
    ButtonRight,
}

/// Process state information from a 2 state switch.
/// [Debouncr](https://github.com/dbrgn/debouncr/) with a 4 sample array is used for debouncing.
pub struct Switch {
    pin: arduino_hal::port::Pin<arduino_hal::port::mode::Input<arduino_hal::port::mode::PullUp>>,
    state: Debouncer<u8, Repeat8>,
    falling: bool,
    rising: bool,
    switch_type: SwitchType,
    double_threshold: Option<u32>,
    held_threshold: Option<u32>,
    held_counter: u32,
    last_press_counter: u32,
    single_press: bool,
    double_press: bool,
}

// @TODO change the InputPin type to one that matches avr_hal
impl Switch {
    /// Create a new Switch.
    pub fn new(
        // pins: &mut arduino_hal::port::Pins,
        pin_name: ButtonName, 
        switch_type: SwitchType) 
        -> Self {
        let dp = arduino_hal::Peripherals::take().unwrap();
        let pins = arduino_hal::pins!(dp);
        Self {
            // This is where you change the pinout for the switches
            pin: match pin_name {
                ButtonName::ButtonA => { pins.d3.into_pull_up_input().downgrade() },
                ButtonName::ButtonB => { pins.a1.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonX => { pins.a0.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonY => { pins.sck.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonL1 => { pins.a1.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonR1 => { pins.d5.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonL2 => { pins.a2.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonR2 => { pins.d0.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonSelect => { pins.miso.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonStart => { pins.d10.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonHome => { pins.mosi.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonShift => { pins.d2.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonUp => { pins.d7.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonDown => { pins.d8.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonLeft => { pins.d6.into_pull_up_input().downgrade() }, 
                ButtonName::ButtonRight => { pins.d9.into_pull_up_input().downgrade() }
            },
            state: debounce_8(true),
            falling: false,
            rising: false,
            switch_type,
            double_threshold: None,
            held_threshold: None,
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
            SwitchType::PullUp => self.pin.is_low(),
            SwitchType::PullDown => self.pin.is_high(),
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

pub fn build_indicators() -> [arduino_hal::port::Pin<arduino_hal::port::mode::Output>; 2] {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    [
        pins.a3.into_output().downgrade(), // Red
        pins.d4.into_output().downgrade(), // Blue
    ]
}

// Write the constructor for the gamepad's switches
pub fn build_gamepad() -> [Switch; 16] {
    // let dp = arduino_hal::Peripherals::take().unwrap();
    // let mut pins = arduino_hal::pins!(dp);
    [
        Switch::new(ButtonName::ButtonA, SwitchType::PullUp),        // Button A
        Switch::new(ButtonName::ButtonB, SwitchType::PullUp),        // Button B
        Switch::new(ButtonName::ButtonX, SwitchType::PullUp),        // Button X
        Switch::new(ButtonName::ButtonY, SwitchType::PullUp),        // Button Y
        Switch::new(ButtonName::ButtonL1, SwitchType::PullUp),       // Button L1
        Switch::new(ButtonName::ButtonR1, SwitchType::PullUp),       // Button R1
        Switch::new(ButtonName::ButtonL2, SwitchType::PullUp),       // Button L2
        Switch::new(ButtonName::ButtonR2, SwitchType::PullUp),       // Button R2
        Switch::new(ButtonName::ButtonSelect, SwitchType::PullUp),   // Button Select
        Switch::new(ButtonName::ButtonStart, SwitchType::PullUp),    // Button Start
        Switch::new(ButtonName::ButtonHome, SwitchType::PullUp),     // Button Home
        Switch::new(ButtonName::ButtonShift, SwitchType::PullUp),    // Button Shift
        Switch::new(ButtonName::ButtonUp, SwitchType::PullUp),       // Button Up
        Switch::new(ButtonName::ButtonDown, SwitchType::PullUp),     // Button Down
        Switch::new(ButtonName::ButtonLeft, SwitchType::PullUp),     // Button Left
        Switch::new(ButtonName::ButtonRight, SwitchType::PullUp),    // Button Right
    ]
}

// Poll the debouncers and update the gamepad's state
pub fn poll_debouncers(gamepad_signals: &mut [Switch; 16]) -> &[Switch; 16] {
    for switch in gamepad_signals.iter_mut() {
        switch.update();
    }
    return gamepad_signals;
}