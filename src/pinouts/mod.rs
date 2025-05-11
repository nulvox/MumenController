//! Pinout configuration module
//! 
//! This module defines the interface for pin configurations and provides
//! implementations for different hardware setups.

use teensy4_bsp::{
    hal::{gpio, iomuxc},
    pins as bsp_pins,
};
use embedded_hal::digital::InputPin;

/// Trait that defines what a pinout configuration must implement
pub trait PinoutConfig {
    /// Function to configure the pins
    fn configure_pins(
        &self,
        pins: &mut bsp_pins::t40::Pins,
        gpio1: &mut gpio::Port<1>,
        gpio2: &mut gpio::Port<2>,
        gpio4: &mut gpio::Port<4>,
    ) -> PinConfig;
    
    /// Whether a particular button/input is configured in this pinout
    fn is_configured(&self, pin_type: PinType) -> bool;
    
    /// Get neutral value for analog inputs (128 is center position)
    fn get_neutral_value(&self, pin_type: PinType) -> u8 {
        match pin_type {
            PinType::Lx | PinType::Ly | PinType::Rx | PinType::Ry => 128,
            _ => 0,
        }
    }
}

/// Type of pin/input
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PinType {
    A,
    B,
    X,
    Y,
    L1,
    R1,
    L2,
    R2,
    L3,
    R3,
    Select,
    Start,
    Home,
    Up,
    Down,
    Left,
    Right,
    AnalogLeft,
    AnalogRight,
    Lock,
    Lx,
    Ly,
    Rx,
    Ry,
}

/// Configuration of pins for the controller
/// Using a simplified approach to avoid ownership issues
pub struct PinConfig {
    // Instead of storing the actual Input types, we'll just track which pins are active
    pub active_pins: u32, // Bit flags for active pins
    // We're not including analog inputs for now, but would be added here
    // pub pin_rx: Option<adc::AnalogInput<pins::t40::P22, 9>>,
    // pub pin_ry: Option<adc::AnalogInput<pins::t40::P23, 10>>,
    // pub pin_lx: Option<adc::AnalogInput<pins::t40::P20, 7>>,
    // pub pin_ly: Option<adc::AnalogInput<pins::t40::P21, 8>>,
}

// Constants for pin bit flags
pub const PIN_A: u32 = 1 << 0;
pub const PIN_B: u32 = 1 << 1;
pub const PIN_X: u32 = 1 << 2;
pub const PIN_Y: u32 = 1 << 3;
pub const PIN_L1: u32 = 1 << 4;
pub const PIN_R1: u32 = 1 << 5;
pub const PIN_L2: u32 = 1 << 6;
pub const PIN_R2: u32 = 1 << 7;
pub const PIN_L3: u32 = 1 << 8;
pub const PIN_R3: u32 = 1 << 9;
pub const PIN_SELECT: u32 = 1 << 10;
pub const PIN_START: u32 = 1 << 11;
pub const PIN_HOME: u32 = 1 << 12;
pub const PIN_UP: u32 = 1 << 13;
pub const PIN_DOWN: u32 = 1 << 14;
pub const PIN_LEFT: u32 = 1 << 15;
pub const PIN_RIGHT: u32 = 1 << 16;
pub const PIN_T_ANALOG_LEFT: u32 = 1 << 17;
pub const PIN_T_ANALOG_RIGHT: u32 = 1 << 18;
pub const PIN_LOCK: u32 = 1 << 19;

/// Helper function to safely check if a pin is low
/// Returns false if the pin is not configured or if there's an error
pub fn is_pin_low(_pin_type: PinType) -> bool {
    // Always return false for this dummy implementation
    false
}

/// Helper function to safely check if a pin is high
/// Returns false if the pin is not configured or if there's an error
pub fn is_pin_high(_pin_type: PinType) -> bool {
    // Always return false for this dummy implementation
    false
}

// Include the available pinout configurations
pub mod standard;
pub mod alternate;

// Factory function to create the configured pinout
#[cfg(feature = "alternate_pinout")]
pub fn create_pinout() -> impl PinoutConfig {
    alternate::AlternatePinout::new()
}

#[cfg(not(feature = "alternate_pinout"))]
pub fn create_pinout() -> impl PinoutConfig {
    standard::StandardPinout::new()
}