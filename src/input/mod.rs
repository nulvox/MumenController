//! Input handling for controller buttons and sticks
//!
//! This module handles all input processing, including digital and analog inputs,
//! debouncing, SOCD handling, and lock logic.

use crate::usb::SwitchProReport;
use log::debug;

mod digital;
mod analog;
mod socd;
mod lock;

pub use digital::{DigitalInputHandler, ControllerButton};
pub use analog::{AnalogInputHandler, AnalogStick};
pub use socd::{SocdHandler, SocdMethod};
pub use lock::{LockHandler, LockableButton};

/// Combined input state for a Nintendo Switch Pro controller
pub struct ControllerState {
    /// Button states (excluding D-pad)
    pub button_states: [bool; 14],
    /// D-pad states (up, down, left, right)
    pub dpad_states: [bool; 4],
    /// Left analog stick position (x, y)
    pub left_stick: (u8, u8),
    /// Right analog stick position (x, y)
    pub right_stick: (u8, u8),
}

impl ControllerState {
    /// Create a new controller state with default values
    pub fn new() -> Self {
        Self {
            button_states: [false; 14],
            dpad_states: [false; 4],
            left_stick: (128, 128),  // Center position
            right_stick: (128, 128), // Center position
        }
    }
    
    /// Convert to a USB HID report
    pub fn to_report(&self) -> SwitchProReport {
        let mut report = SwitchProReport::new();
        
        // Set button states
        for (i, &state) in self.button_states.iter().enumerate() {
            report.set_button(i, state);
        }
        
        // Set D-pad state as HAT value
        let hat = match (self.dpad_states[0], self.dpad_states[3], self.dpad_states[1], self.dpad_states[2]) {
            (true, false, false, false) => 0, // Up
            (true, false, false, true) => 1,  // Up+Right
            (false, false, false, true) => 2, // Right
            (false, true, false, true) => 3,  // Down+Right
            (false, true, false, false) => 4, // Down
            (false, true, true, false) => 5,  // Down+Left
            (false, false, true, false) => 6, // Left
            (true, false, true, false) => 7,  // Up+Left
            _ => 8, // None/Released or invalid combination
        };
        report.set_hat(hat);
        
        // Set analog stick values
        report.left_stick_x = self.left_stick.0;
        report.left_stick_y = self.left_stick.1;
        report.right_stick_x = self.right_stick.0;
        report.right_stick_y = self.right_stick.1;
        
        report
    }
}

/// Input Manager that combines all input handlers
pub struct InputManager {
    /// Digital input handler for buttons
    digital_handler: DigitalInputHandler,
    /// Analog input handler for joysticks
    analog_handler: AnalogInputHandler,
    /// SOCD handler for resolving contradictory inputs
    socd_handler: SocdHandler,
    /// Lock handler for preventing accidental menu button presses
    lock_handler: LockHandler,
    /// Current controller state
    state: ControllerState,
}

impl InputManager {
    /// Create a new input manager with default handlers
    pub fn new() -> Self {
        Self {
            digital_handler: DigitalInputHandler::new(),
            analog_handler: AnalogInputHandler::new(),
            socd_handler: SocdHandler::new(),
            lock_handler: LockHandler::new(),
            state: ControllerState::new(),
        }
    }
    
    /// Create a new input manager with custom handlers
    pub fn with_handlers(
        digital_handler: DigitalInputHandler,
        analog_handler: AnalogInputHandler,
        socd_handler: SocdHandler,
        lock_handler: LockHandler,
    ) -> Self {
        Self {
            digital_handler,
            analog_handler,
            socd_handler,
            lock_handler,
            state: ControllerState::new(),
        }
    }
    
    /// Poll all inputs and update the controller state
    pub fn poll(&mut self, digital_pins: &[bool], analog_values: &[u16], lock_pin: bool) -> &ControllerState {
        // Update lock state
        self.lock_handler.update_lock_state(lock_pin);
        
        // Process digital inputs (returns buttons and dpad separately)
        let (buttons, dpad) = self.digital_handler.update(digital_pins);
        
        // Apply SOCD handling to D-pad inputs
        // The order is (up, down, left, right) for SOCD handler
        // But we need to adjust order for the SocdHandler API which expects (left, right, up, down)
        let (left, right, up, down) = self.socd_handler.resolve(
            dpad[2], // left
            dpad[3], // right
            dpad[0], // up
            dpad[1], // down
        );
        
        // Apply lock functionality to buttons
        let locked_buttons = self.lock_handler.process(&buttons);
        
        // Process analog inputs
        let (left_stick, right_stick) = self.analog_handler.update(analog_values);
        
        // Update the state
        self.state.button_states = locked_buttons;
        self.state.dpad_states = [up, down, left, right]; // Reordered to match expected order
        self.state.left_stick = left_stick;
        self.state.right_stick = right_stick;
        
        debug!("Input poll completed. Buttons: {:?}, D-pad: {:?}",
            self.state.button_states, self.state.dpad_states);
        
        &self.state
    }
    
    /// Get the current controller state
    pub fn get_state(&self) -> &ControllerState {
        &self.state
    }
    
    /// Convert current state to USB HID report
    pub fn to_report(&self) -> SwitchProReport {
        self.state.to_report()
    }
    
    /// Get a reference to the digital input handler
    pub fn get_digital_handler(&self) -> &DigitalInputHandler {
        &self.digital_handler
    }
    
    /// Get a reference to the analog input handler
    pub fn get_analog_handler(&self) -> &AnalogInputHandler {
        &self.analog_handler
    }
    
    /// Get a reference to the SOCD handler
    pub fn get_socd_handler(&self) -> &SocdHandler {
        &self.socd_handler
    }
    
    /// Get a reference to the lock handler
    pub fn get_lock_handler(&self) -> &LockHandler {
        &self.lock_handler
    }
    
    /// Get a mutable reference to the digital input handler
    pub fn get_digital_handler_mut(&mut self) -> &mut DigitalInputHandler {
        &mut self.digital_handler
    }
    
    /// Get a mutable reference to the analog input handler
    pub fn get_analog_handler_mut(&mut self) -> &mut AnalogInputHandler {
        &mut self.analog_handler
    }
    
    /// Get a mutable reference to the SOCD handler
    pub fn get_socd_handler_mut(&mut self) -> &mut SocdHandler {
        &mut self.socd_handler
    }
    
    /// Get a mutable reference to the lock handler
    pub fn get_lock_handler_mut(&mut self) -> &mut LockHandler {
        &mut self.lock_handler
    }
    
    /// Reset the manager to default state
    pub fn reset(&mut self) {
        self.state = ControllerState::new();
        // Additional reset logic for handlers could be added here
    }
}