//! Standard pinout configuration
//! 
//! This is the default configuration matching the original pinout.

use teensy4_bsp::{
    hal::{gpio, iomuxc},
    pins,
};

use super::{PinConfig, PinType, PinoutConfig};

/// Standard pinout configuration
pub struct StandardPinout {
    // Configuration parameters go here if needed
}

impl StandardPinout {
    pub fn new() -> Self {
        StandardPinout {}
    }
}

impl PinoutConfig for StandardPinout {
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
            
        // Lock pin uses pull-down as it needs to be high when active
        let lock_config = iomuxc::Config::zero()
            .set_pull_keeper(Some(iomuxc::PullKeeper::Pulldown22k));
            
        // Apply the configurations to each pin
        iomuxc::configure(&mut pins.p0, lock_config);
        iomuxc::configure(&mut pins.p1, digital_config);
        iomuxc::configure(&mut pins.p2, digital_config);
        iomuxc::configure(&mut pins.p3, digital_config);
        iomuxc::configure(&mut pins.p4, digital_config);
        iomuxc::configure(&mut pins.p5, digital_config);
        iomuxc::configure(&mut pins.p6, digital_config);
        iomuxc::configure(&mut pins.p7, digital_config);
        iomuxc::configure(&mut pins.p8, digital_config);
        iomuxc::configure(&mut pins.p9, digital_config);
        iomuxc::configure(&mut pins.p10, digital_config);
        iomuxc::configure(&mut pins.p11, digital_config);
        iomuxc::configure(&mut pins.p12, digital_config);
        iomuxc::configure(&mut pins.p13, digital_config);
        iomuxc::configure(&mut pins.p14, digital_config);
        iomuxc::configure(&mut pins.p15, digital_config);
        iomuxc::configure(&mut pins.p16, digital_config);
        iomuxc::configure(&mut pins.p17, digital_config);
        iomuxc::configure(&mut pins.p18, digital_config);
        iomuxc::configure(&mut pins.p19, digital_config);
        // Analog pins would be configured here
        // iomuxc::configure(&mut pins.p20, ANALOG_PIN_CONFIG);
        // iomuxc::configure(&mut pins.p21, ANALOG_PIN_CONFIG);
        // iomuxc::configure(&mut pins.p22, ANALOG_PIN_CONFIG);
        // iomuxc::configure(&mut pins.p23, ANALOG_PIN_CONFIG);

        // Create and return the pin configuration
        PinConfig {
            pin_a: Some(gpio1.input(pins.p14)),
            pin_b: Some(gpio2.input(pins.p11)),
            pin_x: Some(gpio2.input(pins.p9)),
            pin_y: Some(gpio1.input(pins.p16)),
            pin_l1: Some(gpio1.input(pins.p15)),
            pin_r1: Some(gpio2.input(pins.p10)),
            pin_l2: Some(gpio2.input(pins.p12)),
            pin_r2: Some(gpio2.input(pins.p13)),
            pin_l3: Some(gpio4.input(pins.p3)),
            pin_r3: Some(gpio4.input(pins.p2)),
            pin_select: Some(gpio1.input(pins.p18)),
            pin_start: Some(gpio1.input(pins.p17)),
            pin_home: Some(gpio2.input(pins.p8)),
            pin_up: Some(gpio1.input(pins.p1)),
            pin_down: Some(gpio2.input(pins.p6)),
            pin_left: Some(gpio2.input(pins.p7)),
            pin_right: Some(gpio1.input(pins.p19)),
            pin_t_analog_left: Some(gpio4.input(pins.p4)),
            pin_t_analog_right: Some(gpio4.input(pins.p5)),
            pin_lock: Some(gpio1.input(pins.p0)),
            // Analog inputs would be initialized here
            // pin_rx: Some(adc::AnalogInput::new(pins.p22)),
            // pin_ry: Some(adc::AnalogInput::new(pins.p23)),
            // pin_lx: Some(adc::AnalogInput::new(pins.p20)),
            // pin_ly: Some(adc::AnalogInput::new(pins.p21)),
        }
    }

    fn is_configured(&self, pin_type: PinType) -> bool {
        // All pins are configured in the standard pinout
        true
    }
}