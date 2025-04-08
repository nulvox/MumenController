//! Pinout configuration module
//! 
//! This module defines the interface for pin configurations and provides
//! implementations for different hardware setups.

use teensy4_bsp::{
    hal::{gpio, iomuxc},
    pins,
};
use embedded_hal::digital::InputPin;

/// Trait that defines what a pinout configuration must implement
pub trait PinoutConfig {
    /// Function to configure the pins
    fn configure_pins(
        &self,
        pins: &mut teensy4_bsp::Pins,
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

/// Configuration of all pins for the controller
pub struct PinConfig {
    pub pin_a: Option<gpio::Input<pins::t40::P14>>,
    pub pin_b: Option<gpio::Input<pins::t40::P11>>,
    pub pin_x: Option<gpio::Input<pins::t40::P9>>,
    pub pin_y: Option<gpio::Input<pins::t40::P16>>,
    pub pin_l1: Option<gpio::Input<pins::t40::P15>>,
    pub pin_r1: Option<gpio::Input<pins::t40::P10>>,
    pub pin_l2: Option<gpio::Input<pins::t40::P12>>,
    pub pin_r2: Option<gpio::Input<pins::t40::P13>>,
    pub pin_l3: Option<gpio::Input<pins::t40::P3>>,
    pub pin_r3: Option<gpio::Input<pins::t40::P2>>,
    pub pin_select: Option<gpio::Input<pins::t40::P18>>,
    pub pin_start: Option<gpio::Input<pins::t40::P17>>,
    pub pin_home: Option<gpio::Input<pins::t40::P8>>,
    pub pin_up: Option<gpio::Input<pins::t40::P1>>,
    pub pin_down: Option<gpio::Input<pins::t40::P6>>,
    pub pin_left: Option<gpio::Input<pins::t40::P7>>,
    pub pin_right: Option<gpio::Input<pins::t40::P19>>,
    pub pin_t_analog_left: Option<gpio::Input<pins::t40::P4>>,
    pub pin_t_analog_right: Option<gpio::Input<pins::t40::P5>>,
    pub pin_lock: Option<gpio::Input<pins::t40::P0>>,
    // We're not including analog inputs for now, but would be added here
    // pub pin_rx: Option<adc::AnalogInput<pins::t40::P22, 9>>,
    // pub pin_ry: Option<adc::AnalogInput<pins::t40::P23, 10>>,
    // pub pin_lx: Option<adc::AnalogInput<pins::t40::P20, 7>>,
    // pub pin_ly: Option<adc::AnalogInput<pins::t40::P21, 8>>,
}

/// Helper function to safely check if a pin is low
/// Returns false if the pin is not configured or if there's an error
pub fn is_pin_low<P: InputPin>(pin_opt: &Option<P>) -> bool {
    match pin_opt {
        Some(pin) => pin.is_low().unwrap_or(false),
        None => false,
    }
}

/// Helper function to safely check if a pin is high
/// Returns false if the pin is not configured or if there's an error
pub fn is_pin_high<P: InputPin>(pin_opt: &Option<P>) -> bool {
    match pin_opt {
        Some(pin) => pin.is_high().unwrap_or(false),
        None => false,
    }
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