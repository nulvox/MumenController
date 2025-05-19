//! Lock pin functionality for controller inputs
//!
//! This module implements the lock pin feature that prevents
//! accidental presses of menu buttons such as Home, Plus, Minus.

use crate::config::PinoutConfig;
use log::debug;

/// Enum for button types that can be locked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockableButton {
    /// The Home button
    Home,
    /// The Plus button (Start)
    Plus,
    /// The Minus button (Select)
    Minus,
    /// The Capture button (for screenshots, etc)
    Capture,
}

impl LockableButton {
    pub fn to_index(&self) -> usize {
        match self {
            LockableButton::Home => 12,    // Index for Home in SwitchProReport
            LockableButton::Plus => 9,     // Index for Plus in SwitchProReport
            LockableButton::Minus => 8,    // Index for Minus in SwitchProReport
            LockableButton::Capture => 13, // Index for Capture in SwitchProReport
        }
    }
    
    pub fn all() -> [LockableButton; 4] {
        [
            LockableButton::Home,
            LockableButton::Plus,
            LockableButton::Minus,
            LockableButton::Capture,
        ]
    }
}

/// Maximum number of lockable buttons
pub const MAX_LOCKABLE_BUTTONS: usize = 4;

/// Lock input handler to prevent accidental menu button presses
pub struct LockHandler {
    /// Current state of the lock pin
    lock_active: bool,
    /// Pin number for the lock switch
    lock_pin: u8,
    /// Whether lock is active (HIGH) or inactive (LOW)
    active_high: bool,
    /// Fixed-size array of buttons that are affected by the lock
    locked_buttons: [Option<LockableButton>; MAX_LOCKABLE_BUTTONS],
    /// Number of active buttons in the locked_buttons array
    button_count: usize,
}

impl LockHandler {
    /// Create a new lock handler in inactive state
    pub fn new() -> Self {
        let mut handler = Self {
            lock_active: false,
            lock_pin: 33, // Default lock pin
            active_high: true, // Lock is active when pin is HIGH
            locked_buttons: [None; MAX_LOCKABLE_BUTTONS],
            button_count: 0,
        };
        
        // Add default locked buttons
        let default_buttons = [
            LockableButton::Home,
            LockableButton::Plus,
            LockableButton::Minus,
        ];
        
        for button in default_buttons {
            handler.add_button(button);
        }
        
        handler
    }
    
    /// Create a new lock handler with specific configuration
    pub fn with_config(lock_pin: u8, active_high: bool, buttons: &[LockableButton]) -> Self {
        let mut handler = Self {
            lock_active: false,
            lock_pin,
            active_high,
            locked_buttons: [None; MAX_LOCKABLE_BUTTONS],
            button_count: 0,
        };
        
        // Add each button from the input slice
        for &button in buttons.iter().take(MAX_LOCKABLE_BUTTONS) {
            handler.add_button(button);
        }
        
        handler
    }
    
    /// Add a button to the locked buttons list
    pub fn add_button(&mut self, button: LockableButton) -> bool {
        if self.button_count < MAX_LOCKABLE_BUTTONS {
            self.locked_buttons[self.button_count] = Some(button);
            self.button_count += 1;
            true
        } else {
            false // Array is full
        }
    }
    
    /// Clear all locked buttons
    pub fn clear_buttons(&mut self) {
        self.locked_buttons = [None; MAX_LOCKABLE_BUTTONS];
        self.button_count = 0;
    }
    
    /// Read the lock pin state
    pub fn read_lock_pin(&self) -> bool {
        // This is a placeholder - in a real implementation, this would
        // read from the GPIO pin using the Teensy BSP
        // For now, we'll just return false (lock inactive)
        
        // Accessing pins would normally involve the MCU's GPIO module
        // For example, something like:
        // let pin_state = gpio.read_pin(self.lock_pin);
        // if self.active_high { pin_state == PinState::High } else { pin_state == PinState::Low }
        
        false
    }
    
    /// Update the lock pin state
    pub fn update_lock_state(&mut self, pin_state: bool) {
        // Convert pin state to lock state based on active high/low configuration
        self.lock_active = if self.active_high { pin_state } else { !pin_state };
        
        debug!("Lock state updated: {}", self.lock_active);
    }
    
    /// Check if the lock is active
    pub fn is_locked(&self) -> bool {
        self.lock_active
    }
    
    /// Process button inputs with lock functionality
    ///
    /// If lock is active, menu buttons (Home, Plus, Minus) are ignored
    /// Returns a new array with the applied lock logic
    pub fn process(&self, button_states: &[bool; 14]) -> [bool; 14] {
        let mut result = *button_states;
        
        // If lock is active, prevent menu button presses
        if self.lock_active {
            for i in 0..self.button_count {
                if let Some(button) = self.locked_buttons[i] {
                    let index = button.to_index();
                    if index < result.len() {
                        result[index] = false;
                    }
                }
            }
            
            debug!("Lock active: Menu buttons suppressed");
        }
        
        result
    }
}