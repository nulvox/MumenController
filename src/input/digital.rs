//! Digital input handling for controller buttons
//!
//! This module handles digital inputs (buttons) with debouncing.

use crate::util::debounce::Debouncer;
use crate::config::PinoutConfig;
use core::convert::TryFrom;
use log::debug;

/// Enum representing the buttons on the Nintendo Switch Pro controller
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerButton {
    A,
    B,
    X,
    Y,
    L,
    R,
    ZL,
    ZR,
    Plus,
    Minus,
    Home,
    Capture,
    L3,
    R3,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
}

/// Map from button enum to index in button array
impl TryFrom<usize> for ControllerButton {
    type Error = ();
    
    fn try_from(index: usize) -> Result<Self, Self::Error> {
        match index {
            0 => Ok(ControllerButton::A),
            1 => Ok(ControllerButton::B),
            2 => Ok(ControllerButton::X),
            3 => Ok(ControllerButton::Y),
            4 => Ok(ControllerButton::L),
            5 => Ok(ControllerButton::R),
            6 => Ok(ControllerButton::ZL),
            7 => Ok(ControllerButton::ZR),
            8 => Ok(ControllerButton::Plus),
            9 => Ok(ControllerButton::Minus),
            10 => Ok(ControllerButton::Home),
            11 => Ok(ControllerButton::Capture),
            12 => Ok(ControllerButton::L3),
            13 => Ok(ControllerButton::R3),
            14 => Ok(ControllerButton::DpadUp),
            15 => Ok(ControllerButton::DpadDown),
            16 => Ok(ControllerButton::DpadLeft),
            17 => Ok(ControllerButton::DpadRight),
            _ => Err(()),
        }
    }
}

/// Map from button name to button index in the SwitchProReport
pub fn button_to_report_index(button: ControllerButton) -> usize {
    match button {
        ControllerButton::A => 0,
        ControllerButton::B => 1,
        ControllerButton::X => 2,
        ControllerButton::Y => 3,
        ControllerButton::L => 4,
        ControllerButton::R => 5,
        ControllerButton::ZL => 6,
        ControllerButton::ZR => 7,
        ControllerButton::Minus => 8,
        ControllerButton::Plus => 9,
        ControllerButton::L3 => 10,
        ControllerButton::R3 => 11,
        ControllerButton::Home => 12,
        ControllerButton::Capture => 13,
        // D-pad buttons are not part of the buttons array in the report
        // They are handled separately via the hat field
        _ => 0, // Default case shouldn't be reached for D-pad
    }
}

/// Digital input handler
pub struct DigitalInputHandler {
    /// Debouncers for each button
    debouncers: [Debouncer; 18], // One debouncer for each button including d-pad
    /// Button mapping from pin to button
    button_mapping: [(u8, ControllerButton); 18],
    /// Current button states
    button_states: [bool; 18],
}

impl DigitalInputHandler {
    /// Create a new digital input handler
    pub fn new() -> Self {
        // Initialize all debouncers in released state
        let debouncers = [
            Debouncer::new(), Debouncer::new(), Debouncer::new(), Debouncer::new(),
            Debouncer::new(), Debouncer::new(), Debouncer::new(), Debouncer::new(),
            Debouncer::new(), Debouncer::new(), Debouncer::new(), Debouncer::new(),
            Debouncer::new(), Debouncer::new(), Debouncer::new(), Debouncer::new(),
            Debouncer::new(), Debouncer::new(),
        ];
        
        // Default mapping from pinout configuration
        let button_mapping = [
            (2, ControllerButton::A),
            (3, ControllerButton::B),
            (4, ControllerButton::X),
            (5, ControllerButton::Y),
            (6, ControllerButton::L),
            (7, ControllerButton::R),
            (8, ControllerButton::ZL),
            (9, ControllerButton::ZR),
            (10, ControllerButton::Plus),
            (11, ControllerButton::Minus),
            (12, ControllerButton::Home),
            (14, ControllerButton::Capture),
            (15, ControllerButton::L3),
            (16, ControllerButton::R3),
            (17, ControllerButton::DpadUp),
            (18, ControllerButton::DpadDown),
            (19, ControllerButton::DpadLeft),
            (20, ControllerButton::DpadRight),
        ];
        
        Self {
            debouncers,
            button_mapping,
            button_states: [false; 18],
        }
    }
    
    /// Read input from a specific pin
    pub fn read_pin(&self, pin: u8) -> bool {
        // This is a placeholder - in a real implementation, this would
        // read from the GPIO pins using the Teensy BSP
        // For now, we'll simulate button presses based on pin number
        
        // Accessing pins would normally involve the MCU's GPIO module
        // For example, something like:
        // gpio.read_pin(pin) == PinState::High
        
        false // Default to not pressed
    }
    
    /// Update button states based on pin readings
    pub fn update(&mut self, pins_state: &[bool]) -> ([bool; 14], [bool; 4]) {
        let mut standard_buttons = [false; 14]; // Non-dpad buttons for report
        let mut dpad = [false; 4]; // Up, Down, Left, Right
        
        // Update each button's state with debouncing
        for (i, (pin, button)) in self.button_mapping.iter().enumerate() {
            // Read the pin state from the passed array if it's within range,
            // otherwise default to false (not pressed)
            let pin_value = if (*pin as usize) < pins_state.len() {
                pins_state[*pin as usize]
            } else {
                false
            };
            
            // Apply debouncing
            let debounced_state = self.debouncers[i].update(pin_value);
            self.button_states[i] = debounced_state;
            
            // Map to appropriate output array
            match button {
                ControllerButton::DpadUp => dpad[0] = debounced_state,
                ControllerButton::DpadDown => dpad[1] = debounced_state,
                ControllerButton::DpadLeft => dpad[2] = debounced_state,
                ControllerButton::DpadRight => dpad[3] = debounced_state,
                _ => {
                    let index = button_to_report_index(*button);
                    if index < standard_buttons.len() {
                        standard_buttons[index] = debounced_state;
                    }
                }
            }
        }
        
        (standard_buttons, dpad)
    }
    
    /// Get raw button states without updating
    pub fn get_raw_states(&self) -> [bool; 18] {
        self.button_states
    }
    
    /// Get dpad states as a tuple (up, down, left, right)
    pub fn get_dpad_states(&self) -> (bool, bool, bool, bool) {
        let up = self.button_states[14]; // Index for DpadUp
        let down = self.button_states[15]; // Index for DpadDown
        let left = self.button_states[16]; // Index for DpadLeft
        let right = self.button_states[17]; // Index for DpadRight
        
        (up, down, left, right)
    }
}