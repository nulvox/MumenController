//! Analog input handling for controller sticks
//!
//! This module handles analog inputs (joysticks) including filtering and calibration.

use crate::config::PinoutConfig;
use log::debug;

/// Represents an analog stick with X and Y axes
#[derive(Debug, Clone, Copy)]
pub enum AnalogStick {
    Left,
    Right,
}

/// Analog input handler
pub struct AnalogInputHandler {
    // Calibration values for left stick
    left_center_x: u16,
    left_center_y: u16,
    left_min_x: u16,
    left_min_y: u16,
    left_max_x: u16,
    left_max_y: u16,
    
    // Calibration values for right stick
    right_center_x: u16,
    right_center_y: u16,
    right_min_x: u16,
    right_min_y: u16,
    right_max_x: u16,
    right_max_y: u16,
    
    // Deadzone values (in ADC units)
    deadzone: u16,
    
    // Filter values for smoothing
    filter_alpha: f32,
    left_filtered_x: f32,
    left_filtered_y: f32,
    right_filtered_x: f32,
    right_filtered_y: f32,
}

impl AnalogInputHandler {
    /// Create a new analog input handler with default calibration
    pub fn new() -> Self {
        Self {
            // Default calibration values (middle of 10-bit ADC range)
            left_center_x: 512,
            left_center_y: 512,
            left_min_x: 0,
            left_min_y: 0,
            left_max_x: 1023,
            left_max_y: 1023,
            
            right_center_x: 512,
            right_center_y: 512,
            right_min_x: 0,
            right_min_y: 0,
            right_max_x: 1023,
            right_max_y: 1023,
            
            // Default deadzone (about 5% of full range)
            deadzone: 50,
            
            // Default filter alpha (lower = more filtering)
            filter_alpha: 0.3,
            left_filtered_x: 512.0,
            left_filtered_y: 512.0,
            right_filtered_x: 512.0,
            right_filtered_y: 512.0,
        }
    }
    
    /// Configure deadzone size
    pub fn set_deadzone(&mut self, deadzone: u16) {
        self.deadzone = deadzone;
    }
    
    /// Configure filter strength (0.0 = max filtering, 1.0 = no filtering)
    pub fn set_filter_strength(&mut self, alpha: f32) {
        self.filter_alpha = alpha.max(0.0).min(1.0);
    }
    
    /// Read analog input from a specific pin
    pub fn read_analog_pin(&self, pin: u8) -> u16 {
        // This is a placeholder - in a real implementation, this would
        // read from the ADC pins using the Teensy BSP
        // For now, we'll simulate joystick positions with a default value
        
        // Accessing ADC would normally involve the MCU's ADC module
        // For example, something like:
        // adc.read_pin(pin)
        
        512 // Default to center position
    }
    
    /// Calibrate center position for a stick
    pub fn calibrate_center(&mut self, stick: AnalogStick) {
        // In a real implementation, this would read the current position
        // and set it as the center. For now, we'll just use default values.
        match stick {
            AnalogStick::Left => {
                self.left_center_x = 512;
                self.left_center_y = 512;
            },
            AnalogStick::Right => {
                self.right_center_x = 512;
                self.right_center_y = 512;
            },
        }
    }
    
    /// Calibrate min/max for a stick (would normally be called during a full stick rotation)
    pub fn calibrate_range(&mut self, stick: AnalogStick, x: u16, y: u16) {
        match stick {
            AnalogStick::Left => {
                self.left_min_x = self.left_min_x.min(x);
                self.left_min_y = self.left_min_y.min(y);
                self.left_max_x = self.left_max_x.max(x);
                self.left_max_y = self.left_max_y.max(y);
            },
            AnalogStick::Right => {
                self.right_min_x = self.right_min_x.min(x);
                self.right_min_y = self.right_min_y.min(y);
                self.right_max_x = self.right_max_x.max(x);
                self.right_max_y = self.right_max_y.max(y);
            },
        }
    }
    
    /// Apply exponential filter to smooth out readings
    fn apply_filter(&mut self, stick: AnalogStick, x: f32, y: f32) -> (f32, f32) {
        match stick {
            AnalogStick::Left => {
                // Apply low-pass filter
                self.left_filtered_x = self.left_filtered_x * (1.0 - self.filter_alpha) + x * self.filter_alpha;
                self.left_filtered_y = self.left_filtered_y * (1.0 - self.filter_alpha) + y * self.filter_alpha;
                (self.left_filtered_x, self.left_filtered_y)
            },
            AnalogStick::Right => {
                // Apply low-pass filter
                self.right_filtered_x = self.right_filtered_x * (1.0 - self.filter_alpha) + x * self.filter_alpha;
                self.right_filtered_y = self.right_filtered_y * (1.0 - self.filter_alpha) + y * self.filter_alpha;
                (self.right_filtered_x, self.right_filtered_y)
            },
        }
    }
    
    /// Apply deadzone to analog input
    fn apply_deadzone(&self, value: i32, center: i32) -> i32 {
        let offset = value - center;
        let deadzone = self.deadzone as i32;
        
        // If input is within deadzone, return center
        if offset.abs() < deadzone {
            center
        } else {
            // Otherwise, scale the input to remove the deadzone jump
            let sign = if offset >= 0 { 1 } else { -1 };
            center + sign * (offset.abs() - deadzone)
        }
    }
    
    /// Process raw analog input for left stick and convert to controller range (0-255)
    pub fn process_left_stick(&mut self, raw_x: u16, raw_y: u16) -> (u8, u8) {
        self.process_input(AnalogStick::Left, raw_x, raw_y)
    }
    
    /// Process raw analog input for right stick and convert to controller range (0-255)
    pub fn process_right_stick(&mut self, raw_x: u16, raw_y: u16) -> (u8, u8) {
        self.process_input(AnalogStick::Right, raw_x, raw_y)
    }
    
    /// Process raw analog input and convert to controller range (0-255)
    fn process_input(&mut self, stick: AnalogStick, raw_x: u16, raw_y: u16) -> (u8, u8) {
        let (center_x, center_y, min_x, min_y, max_x, max_y) = match stick {
            AnalogStick::Left => (
                self.left_center_x,
                self.left_center_y,
                self.left_min_x,
                self.left_min_y,
                self.left_max_x,
                self.left_max_y,
            ),
            AnalogStick::Right => (
                self.right_center_x,
                self.right_center_y,
                self.right_min_x,
                self.right_min_y,
                self.right_max_x,
                self.right_max_y,
            ),
        };
        
        // Convert to integers for easier processing
        let raw_x_i32 = raw_x as i32;
        let raw_y_i32 = raw_y as i32;
        let center_x_i32 = center_x as i32;
        let center_y_i32 = center_y as i32;
        
        // Apply deadzone
        let x_with_deadzone = self.apply_deadzone(raw_x_i32, center_x_i32);
        let y_with_deadzone = self.apply_deadzone(raw_y_i32, center_y_i32);
        
        // Apply filtering (convert to float for filter processing)
        let (filtered_x, filtered_y) = self.apply_filter(
            stick,
            x_with_deadzone as f32,
            y_with_deadzone as f32
        );
        
        // Convert back to integer for range mapping
        let x_filtered_i32 = filtered_x as i32;
        let y_filtered_i32 = filtered_y as i32;
        
        // Map to the controller range (0-255, with 128 as center)
        // We need to handle each quadrant separately to account for asymmetric ranges
        
        // X-axis mapping
        let mapped_x = if x_filtered_i32 < center_x_i32 {
            // Left half of range
            let range = center_x_i32 - min_x as i32;
            if range == 0 {
                128 // Avoid division by zero
            } else {
                128 - ((center_x_i32 - x_filtered_i32) * 128 / range) as u8
            }
        } else {
            // Right half of range
            let range = max_x as i32 - center_x_i32;
            if range == 0 {
                128 // Avoid division by zero
            } else {
                128 + ((x_filtered_i32 - center_x_i32) * 127 / range) as u8
            }
        };
        
        // Y-axis mapping (inverted because lower ADC values typically mean higher stick position)
        let mapped_y = if y_filtered_i32 < center_y_i32 {
            // Upper half of range (lower values in ADC)
            let range = center_y_i32 - min_y as i32;
            if range == 0 {
                128 // Avoid division by zero
            } else {
                128 - ((center_y_i32 - y_filtered_i32) * 128 / range) as u8
            }
        } else {
            // Lower half of range (higher values in ADC)
            let range = max_y as i32 - center_y_i32;
            if range == 0 {
                128 // Avoid division by zero
            } else {
                128 + ((y_filtered_i32 - center_y_i32) * 127 / range) as u8
            }
        };
        
        // Ensure values are in range 0-255
        let final_x = mapped_x.clamp(0, 255);
        let final_y = mapped_y.clamp(0, 255);
        
        (final_x, final_y)
    }
    
    /// Update all stick readings and return the processed values
    pub fn update(&mut self, adc_values: &[u16]) -> ((u8, u8), (u8, u8)) {
        // For a real implementation, we would read from the ADC pins
        // For now, we'll use the provided values or defaults
        
        // Use provided values or defaults if not available
        let left_x = if adc_values.len() > 0 { adc_values[0] } else { 512 };
        let left_y = if adc_values.len() > 1 { adc_values[1] } else { 512 };
        let right_x = if adc_values.len() > 2 { adc_values[2] } else { 512 };
        let right_y = if adc_values.len() > 3 { adc_values[3] } else { 512 };
        
        let left_stick = self.process_left_stick(left_x, left_y);
        let right_stick = self.process_right_stick(right_x, right_y);
        
        (left_stick, right_stick)
    }
}