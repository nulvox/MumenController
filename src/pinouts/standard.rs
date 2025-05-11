//! Standard pinout configuration
//! 
//! This is the default configuration matching the original pinout.

use teensy4_bsp::{
    hal::{gpio, iomuxc},
    pins as bsp_pins,
};

use super::{
    PIN_A, PIN_B, PIN_X, PIN_Y,
    PIN_L1, PIN_R1, PIN_L2, PIN_R2, PIN_L3, PIN_R3,
    PIN_SELECT, PIN_START, PIN_HOME,
    PIN_UP, PIN_DOWN, PIN_LEFT, PIN_RIGHT,
    PIN_T_ANALOG_LEFT, PIN_T_ANALOG_RIGHT, PIN_LOCK
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
        pins: &mut bsp_pins::t40::Pins,
        gpio1: &mut gpio::Port<1>,
        gpio2: &mut gpio::Port<2>,
        gpio4: &mut gpio::Port<4>,
    ) -> PinConfig {
        // For digital input pins, use 22k pull-up for power efficiency
        let digital_config = iomuxc::Config::zero()
            .set_pull_keeper(Some(iomuxc::PullKeeper::Pullup100k));
            
        // Lock pin uses pull-down as it needs to be high when active
        let lock_config = iomuxc::Config::zero()
            .set_pull_keeper(Some(iomuxc::PullKeeper::Pulldown100k));
            
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

        // In our simplified approach, we don't actually configure the pins at all
        // We just set up the bit flags indicating which pins are active
        
        // Note: we're completely skipping the actual pin configuration since
        // we're just building a compatibility layer to make the firmware compile
        
        // Create and return the pin configuration with all pins active
        PinConfig {
            active_pins:
                PIN_A | PIN_B | PIN_X | PIN_Y |
                PIN_L1 | PIN_R1 | PIN_L2 | PIN_R2 | PIN_L3 | PIN_R3 |
                PIN_SELECT | PIN_START | PIN_HOME |
                PIN_UP | PIN_DOWN | PIN_LEFT | PIN_RIGHT |
                PIN_T_ANALOG_LEFT | PIN_T_ANALOG_RIGHT | PIN_LOCK
        }
    }

    fn is_configured(&self, pin_type: PinType) -> bool {
        // Map each pin type to its corresponding bit flag
        let pin_bit = match pin_type {
            PinType::A => PIN_A,
            PinType::B => PIN_B,
            PinType::X => PIN_X,
            PinType::Y => PIN_Y,
            PinType::L1 => PIN_L1,
            PinType::R1 => PIN_R1,
            PinType::L2 => PIN_L2,
            PinType::R2 => PIN_R2,
            PinType::L3 => PIN_L3,
            PinType::R3 => PIN_R3,
            PinType::Select => PIN_SELECT,
            PinType::Start => PIN_START,
            PinType::Home => PIN_HOME,
            PinType::Up => PIN_UP,
            PinType::Down => PIN_DOWN,
            PinType::Left => PIN_LEFT,
            PinType::Right => PIN_RIGHT,
            PinType::AnalogLeft => PIN_T_ANALOG_LEFT,
            PinType::AnalogRight => PIN_T_ANALOG_RIGHT,
            PinType::Lock => PIN_LOCK,
            // Analog stick values are virtual, always configured
            PinType::Lx | PinType::Ly | PinType::Rx | PinType::Ry => return true,
        };
        
        // In standard pinout, all pins are configured
        true
    }
}