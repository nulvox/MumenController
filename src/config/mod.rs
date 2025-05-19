//! Configuration system for the Nintendo Switch Pro controller
//!
//! This module provides a zero-runtime-overhead, compile-time configuration system
//! for the controller, including pinout configuration, SOCD handling rules,
//! and feature flags. All configuration values are computed at compile time
//! and baked into the firmware binary, making them extremely efficient.
//!
//! ## Usage
//!
//! Access digital pin constants directly:
//! ```rust
//! use crate::config::digital_pins::BUTTON_A;
//! ```
//!
//! Or access pins by name via the static getter methods:
//! ```rust
//! use crate::config::PinoutConfig;
//! let button_pins = PinoutConfig::get_digital_pins();
//! ```
//!
//! Accessing SOCD handling methods:
//! ```rust
//! use crate::config::SocdConfig;
//! let method = SocdConfig::get_method_for_pair("left_right");
//! ```

// Import required dependencies for no_std environment
extern crate alloc;
use core::fmt;

// Re-export the generated configuration
pub mod generated;

/// Constants for default configurations
pub const DEFAULT_PINOUT_CONFIG: &str = "default";
pub const DEFAULT_SOCD_CONFIG: &str = "default";

// Re-export generated configurations
pub use generated::*;

/// Error type for configuration-related errors
#[derive(Debug)]
pub enum ConfigError {
    /// A required pin is missing from the configuration
    MissingPin(&'static str),
    /// A required SOCD rule is missing
    MissingRule(&'static str),
    /// Invalid configuration value
    InvalidValue(&'static str, &'static str),
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::MissingPin(pin) => write!(f, "Missing required pin: {}", pin),
            ConfigError::MissingRule(rule) => write!(f, "Missing required SOCD rule: {}", rule),
            ConfigError::InvalidValue(name, value) => write!(f, "Invalid value for {}: {}", name, value),
        }
    }
}

// Additional utility functions for working with configurations

/// Get a digital pin number by name, returns None if the pin is not found
pub fn get_digital_pin_by_name(name: &str) -> Option<u8> {
    PinoutConfig::get_digital_pins()
        .iter()
        .find(|(pin_name, _)| *pin_name == name)
        .map(|(_, pin)| *pin)
}

/// Get an analog pin number by name, returns None if the pin is not found
pub fn get_analog_pin_by_name(name: &str) -> Option<u8> {
    PinoutConfig::get_analog_pins()
        .iter()
        .find(|(pin_name, _)| *pin_name == name)
        .map(|(_, pin)| *pin)
}

/// Get a special pin number by name, returns None if the pin is not found
pub fn get_special_pin_by_name(name: &str) -> Option<u8> {
    PinoutConfig::get_special_pins()
        .iter()
        .find(|(pin_name, _)| *pin_name == name)
        .map(|(_, pin)| *pin)
}

/// Resolve a SOCD conflict using the configured resolution method
///
/// # Arguments
/// * `input1` - First input direction (e.g., "left", "up")
/// * `input2` - Second input direction (e.g., "right", "down")
///
/// # Returns
/// The resolved direction as a string (e.g., "neutral", "left", "right")
pub fn resolve_socd_conflict(input1: &str, input2: &str) -> &'static str {
    // Instead of creating dynamic strings, use predefined pair names
    let pair_name = if input1 < input2 {
        match (input1, input2) {
            ("left", "right") => "left_right",
            ("up", "down") => "up_down",
            _ => "unknown_pair" // Fallback for other combinations
        }
    } else {
        match (input2, input1) {
            ("left", "right") => "left_right",
            ("up", "down") => "up_down",
            _ => "unknown_pair" // Fallback for other combinations
        }
    };
    
    // Look for custom overrides first for known pairs
    for (combo, result) in SocdConfig::get_custom_overrides() {
        if *combo == pair_name {
            return result;
        }
    }
    
    // Use the standard resolution method
    SocdConfig::get_method_for_pair(pair_name)
}