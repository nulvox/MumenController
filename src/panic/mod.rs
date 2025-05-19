//! Panic handler with LED-based error reporting
//!
//! This module provides visual feedback via the onboard LED when errors occur,
//! with different blink patterns for different error types.

mod led;

pub use led::*;

/// Debug blink patterns for initialization stages
///
/// This function can be used to determine where in the initialization sequence
/// a failure is occurring by blinking the LED a specific number of times.
/// Call this at critical points during initialization to visually show progress.
pub fn debug_blink_stage(led: &mut teensy4_bsp::board::Led, stage: u8) {
    // First turn off LED to ensure we start from a known state
    led.set();
    
    // Simple delay implementation
    let delay_ms = |ms: u32| {
        let cycles_per_ms = teensy4_bsp::board::ARM_FREQUENCY / 1000;
        cortex_m::asm::delay(ms * cycles_per_ms);
    };
    
    // Blink the LED the specified number of times to indicate the stage
    for _ in 0..stage {
        led.clear();
        delay_ms(100);
        led.set();
        delay_ms(100);
    }
    
    // Longer delay to separate stages
    delay_ms(500);
}

// Error types for the panic handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    HardFault,
    MemoryError,
    UsbError,
    InitError,
    ConfigError,
    Other,
}

// Get a string representation of the error type
pub fn error_type_name(error_type: ErrorType) -> &'static str {
    match error_type {
        ErrorType::HardFault => "Hard Fault",
        ErrorType::MemoryError => "Memory Error",
        ErrorType::UsbError => "USB Error",
        ErrorType::InitError => "Init Error",
        ErrorType::ConfigError => "Config Error",
        ErrorType::Other => "Other Error",
    }
}

// Try to infer error type from panic message
pub fn infer_error_type(message: &str) -> ErrorType {
    if message.contains("memory") || message.contains("allocation") {
        ErrorType::MemoryError
    } else if message.contains("usb") || message.contains("USB") {
        ErrorType::UsbError
    } else if message.contains("init") || message.contains("initialization") {
        ErrorType::InitError
    } else if message.contains("config") || message.contains("configuration") {
        ErrorType::ConfigError
    } else if message.contains("fault") || message.contains("Fault") {
        ErrorType::HardFault
    } else {
        ErrorType::Other
    }
}