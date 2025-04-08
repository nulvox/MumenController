//! Alternate pinout configuration
//! 
//! This configuration has the following changes from standard:
//! - A and B buttons are inverted
//! - L2, R2, L3, and R3 are not configured
//! - Shift and Lock are not configured
//! - The analog sticks (Lx, Ly, Rx, and Ry) are not configured

use teensy4_bsp::{
    hal::{gpio, iomuxc},
    pins,
};

use super::{PinConfig, PinType, PinoutConfig};

/// Alternate pinout configuration
pub struct AlternatePinout {
    // Configuration parameters go here if needed
}

impl AlternatePinout {
    pub fn new() -> Self {
        AlternatePinout {}
    }
}

impl PinoutConfig for AlternatePinout {
    fn configure_pins(
        &self,
        pins: &mut teensy4_bsp::Pins,
        gpio1: &mut gpio::Port<1>,
        gpio2: &mut gpio::Port<2>,
        gpio4: &mut gpio::Port<4>,
    ) -> PinConfig {
        // For digital input pins, use 22k pull-up for power efficiency
        let digital_config = iomuxc::Config::zero()
            .set_pull_keeper(Some(iomuxc::PullKeeper::Pullup22k));
            
        // Configure only the pins we're using
        iomuxc::configure(&mut pins.p1, digital_config);
        iomuxc::configure(&mut pins.p6, digital_config);
        iomuxc::configure(&mut pins.p7, digital_config);
        iomuxc::configure(&mut pins.p8, digital_config);
        iomuxc::configure(&mut pins.p9, digital_config);
        // Note: A and B are inverted, so we configure p11 for A and p14 for B
        iomuxc::configure(&mut pins.p11, digital_config);
        iomuxc::configure(&mut pins.p14, digital_config);
        iomuxc::configure(&mut pins.p15, digital_config);
        iomuxc::configure(&mut pins.p16, digital_config);
        iomuxc::configure(&mut pins.p17, digital_config);
        iomuxc::configure(&mut pins.p18, digital_config);
        iomuxc::configure(&mut pins.p19, digital_config);
        iomuxc::configure(&mut pins.p10, digital_config);

        // Create and return the pin configuration with inverted A/B and missing pins
        PinConfig {
            // A and B buttons are inverted (swapped pins)
            pin_a: Some(gpio2.input(pins.p11)),  // This was pin_b in standard
            pin_b: Some(gpio1.input(pins.p14)),  // This was pin_a in standard
            pin_x: Some(gpio2.input(pins.p9)),
            pin_y: Some(gpio1.input(pins.p16)),
            pin_l1: Some(gpio1.input(pins.p15)),
            pin_r1: Some(gpio2.input(pins.p10)),
            // L2, R2, L3, R3 are not configured
            pin_l2: None,
            pin_r2: None,
            pin_l3: None,
            pin_r3: None,
            pin_select: Some(gpio1.input(pins.p18)),
            pin_start: Some(gpio1.input(pins.p17)),
            pin_home: Some(gpio2.input(pins.p8)),
            pin_up: Some(gpio1.input(pins.p1)),
            pin_down: Some(gpio2.input(pins.p6)),
            pin_left: Some(gpio2.input(pins.p7)),
            pin_right: Some(gpio1.input(pins.p19)),
            // Shift (analog toggles) and Lock are not configured
            pin_t_analog_left: None,
            pin_t_analog_right: None,
            pin_lock: None,
            // Analog inputs are not configured in this pinout
        }
    }

    fn is_configured(&self, pin_type: PinType) -> bool {
        match pin_type {
            // Not configured in this pinout
            PinType::L2 => false,
            PinType::R2 => false,
            PinType::L3 => false,
            PinType::R3 => false,
            PinType::AnalogLeft => false,
            PinType::AnalogRight => false,
            PinType::Lock => false,
            PinType::Lx => false,
            PinType::Ly => false,
            PinType::Rx => false,
            PinType::Ry => false,
            // All other pins are configured
            _ => true,
        }
    }
}