use debouncr::{debounce_8, Debouncer, Edge, Repeat8};
// use debouncer::debounce_8;
// use arduino_hal::port;
use arduino_hal;
use crate::report;
// use report::KeyData;

// Define the array offsets for each switch
pub static SwitchA: usize = 0;
pub static SwitchB: usize = 1;
pub static SwitchX: usize = 2;
pub static SwitchY: usize = 3;
pub static SwitchL1: usize = 4;
pub static SwitchR1: usize = 5;
pub static SwitchL2: usize = 6;
pub static SwitchR2: usize = 7;
pub static SwitchSelect: usize = 8;
pub static SwitchStart: usize = 9;
pub static SwitchHome: usize = 10;
pub static SwitchShift: usize = 11;
pub static SwitchUp: usize = 12;
pub static SwitchDown: usize = 13;
pub static SwitchLeft: usize = 14;
pub static SwitchRight: usize = 15;

/// If the switch is a pull-up or pull-down type
pub enum SwitchType {
    PullUp,
    PullDown,
}

/// Process state information from a 2 state switch.
/// [Debouncr](https://github.com/dbrgn/debouncr/) with a 4 sample array is used for debouncing.
pub struct Switch {
    pin: arduino_hal::port::Pin<arduino_hal::port::mode::Input<arduino_hal::port::mode::Floating>>,
    state: Debouncer<u8, Repeat8>,
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
impl Switch {
    /// Create a new Switch.
    pub fn new(
        pin: arduino_hal::port::Pin<arduino_hal::port::mode::Input<arduino_hal::port::mode::Floating>>, 
        switch_type: SwitchType) 
        -> Self {
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

// @TODO the remaining functions in this file should be a trait implemented for GamePad
// Write the constructor for the gamepad's switches
pub fn build_gamepad(pins: &[arduino_hal::port::Pin<arduino_hal::port::mode::Input<arduino_hal::port::mode::Floating>>; 16]) -> [Switch; 16] {
    let mut switch_array = [
        Switch::new(pins[SwitchA], SwitchType::PullUp),
        Switch::new(pins[SwitchB], SwitchType::PullUp),
        Switch::new(pins[SwitchX], SwitchType::PullUp),
        Switch::new(pins[SwitchY], SwitchType::PullUp),
        Switch::new(pins[SwitchL1], SwitchType::PullUp),
        Switch::new(pins[SwitchR1], SwitchType::PullUp),
        Switch::new(pins[SwitchL2], SwitchType::PullUp),
        Switch::new(pins[SwitchR2], SwitchType::PullUp),
        Switch::new(pins[SwitchSelect], SwitchType::PullUp),
        Switch::new(pins[SwitchStart], SwitchType::PullUp),
        Switch::new(pins[SwitchHome], SwitchType::PullUp),
        Switch::new(pins[SwitchShift], SwitchType::PullUp),
        Switch::new(pins[SwitchUp], SwitchType::PullUp),
        Switch::new(pins[SwitchDown], SwitchType::PullUp),
        Switch::new(pins[SwitchLeft], SwitchType::PullUp),
        Switch::new(pins[SwitchRight], SwitchType::PullUp),
    ];
    return switch_array;
}

// Poll the debouncers and update the gamepad's state
pub fn poll_debouncers(gamepad_signals: &[Switch; 16]) -> [Switch; 16] {
    for switch in gamepad_signals {
        switch.update();
    }
    return *gamepad_signals;
}
