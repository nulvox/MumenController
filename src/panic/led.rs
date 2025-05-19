//! LED-based error reporting system
//!
//! This module implements different blink patterns for the onboard LED
//! to signal different types of errors.

use teensy4_bsp::board::Led;
use crate::panic::ErrorType;

/// LED Error Blinker for visual error feedback
pub struct LedErrorBlinker {
    led: Led,
    error_type: ErrorType,
}

impl LedErrorBlinker {
    /// Create a new LED error blinker
    pub fn new(led: Led, error_type: ErrorType) -> Self {
        Self { led, error_type }
    }

    /// Start blinking the LED with the pattern for the error type.
    /// This function does not return, as it's intended to be used
    /// in panic situations.
    /// Start blinking the LED with the pattern for the error type.
    /// This function does not return, as it's intended to be used
    /// in panic situations.
    ///
    /// IMPROVEMENT: Updated LED error patterns for clearer diagnostics
    /// The patterns have been standardized to be more distinguishable
    /// from each other, making it easier to identify error conditions
    /// by visual inspection without additional tools.
    pub fn start_blink_pattern(&mut self) -> ! {
        match self.error_type {
            ErrorType::HardFault => self.blink_pattern_hard_fault(),
            ErrorType::MemoryError => self.blink_pattern_memory_error(),
            ErrorType::UsbError => self.blink_pattern_usb_error(),
            ErrorType::InitError => self.blink_pattern_init_error(),
            ErrorType::ConfigError => self.blink_pattern_config_error(),
            ErrorType::Other => self.blink_pattern_sos(),
        }
    }

    // Private helper to delay a specific number of milliseconds
    fn delay_ms(&self, ms: u32) {
        // Simple busy wait - this is for panic situations only
        let cycles_per_ms = teensy4_bsp::board::ARM_FREQUENCY / 1000;
        cortex_m::asm::delay(ms * cycles_per_ms);
    }

    // Short blink (200ms on, 200ms off)
    fn blink_short(&mut self) {
        self.led.set();
        self.delay_ms(200);
        self.led.clear();
        self.delay_ms(200);
    }

    // Long blink (600ms on, 200ms off)
    fn blink_long(&mut self) {
        self.led.set();
        self.delay_ms(600);
        self.led.clear();
        self.delay_ms(200);
    }

    // Pattern for Hard Fault: Rapid blinks (5Hz)
    fn blink_pattern_hard_fault(&mut self) -> ! {
        // Initial delay to distinguish the beginning of the pattern
        self.delay_ms(700);
        
        loop {
            self.led.set();
            self.delay_ms(200);
            self.led.clear();
            self.delay_ms(200);
        }
    }

    // Pattern for Memory Error: Long-short-short
    fn blink_pattern_memory_error(&mut self) -> ! {
        // Initial delay to distinguish the beginning of the pattern
        self.delay_ms(700);
        
        loop {
            self.blink_long();
            self.blink_short();
            self.blink_short();
            self.delay_ms(1000); // Pause between pattern repetitions
        }
    }

    // Pattern for USB Error: Long-short-long
    fn blink_pattern_usb_error(&mut self) -> ! {
        // Initial delay to distinguish the beginning of the pattern
        self.delay_ms(700);
        
        loop {
            self.blink_long();
            self.blink_short();
            self.blink_long();
            self.delay_ms(1000); // Pause between pattern repetitions
        }
    }

    // Pattern for Init Error: Continuous on
    /// IMPROVEMENT: Consistent error pattern for initialization failures
    /// The initialization error pattern uses 3 long blinks to make it
    /// easily distinguishable from other error types and consistent
    /// with the documentation.
    fn blink_pattern_init_error(&mut self) -> ! {
        // Initial delay to distinguish the beginning of the pattern
        self.delay_ms(700);
        
        loop {
            // 3 long blinks pattern for InitError
            self.blink_long();
            self.blink_long();
            self.blink_long();
            self.delay_ms(1000); // Pause between pattern repetitions
        }
    }

    // Pattern for Config Error: Short-long-short
    fn blink_pattern_config_error(&mut self) -> ! {
        // Initial delay to distinguish the beginning of the pattern
        self.delay_ms(700);
        
        loop {
            self.blink_short();
            self.blink_long();
            self.blink_short();
            self.delay_ms(1000); // Pause between pattern repetitions
        }
    }

    // SOS pattern (... --- ...) for Other errors
    fn blink_pattern_sos(&mut self) -> ! {
        // Initial delay to distinguish the beginning of the pattern
        self.delay_ms(700);
        
        loop {
            // S (...)
            for _ in 0..3 {
                self.blink_short();
            }
            self.delay_ms(200);
            
            // O (---)
            for _ in 0..3 {
                self.blink_long();
            }
            self.delay_ms(200);
            
            // S (...)
            for _ in 0..3 {
                self.blink_short();
            }
            
            self.delay_ms(1000); // Pause between pattern repetitions
        }
    }
}