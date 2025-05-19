//! Custom panic handler for Teensy 4.0
//!
//! This module provides a bare-metal panic handler that uses raw assembly
//! to control the LED and show code section and error information.

use core::panic::PanicInfo;

// Code section constants to track where in the code the panic occurred
pub const SECTION_NONE: u8 = 0;              // Unknown section
pub const SECTION_INIT: u8 = 1;              // Initialization section
pub const SECTION_PRE_USB: u8 = 2;           // Pre-USB initialization
pub const SECTION_USB_INIT: u8 = 3;          // USB initialization
pub const SECTION_USB_POLL: u8 = 4;          // USB polling section
pub const SECTION_HID_REPORT: u8 = 5;        // HID report section
pub const SECTION_KEY_HANDLING: u8 = 6;      // Key handling section
pub const SECTION_PIN_INIT: u8 = 7;          // Pin initialization
pub const SECTION_GPIO_INIT: u8 = 8;         // GPIO initialization
pub const SECTION_TIMER_INIT: u8 = 9;        // Timer initialization
pub const SECTION_LED_TEST: u8 = 10;         // LED test section
pub const SECTION_MAIN_LOOP: u8 = 11;        // Main loop

// Error type constants
pub const ERR_NONE: u8 = 0;                  // No specific error
pub const ERR_PIN_CONFIG: u8 = 1;            // Pin configuration error
pub const ERR_USB_INIT: u8 = 2;              // USB initialization error
pub const ERR_USB_POLL: u8 = 3;              // USB polling error
pub const ERR_HID_REPORT: u8 = 4;            // HID report error
pub const ERR_KEY_HANDLING: u8 = 5;          // Key handling error
pub const ERR_MEMORY: u8 = 6;                // Memory error
pub const ERR_TIMER: u8 = 7;                 // Timer error
pub const ERR_GPIO_CONFIG: u8 = 8;           // GPIO configuration error
pub const ERR_PINOUT_CONFIG: u8 = 9;         // Pinout configuration error
pub const ERR_USB_DEVICE: u8 = 10;           // USB device error
pub const ERR_USB_BUS: u8 = 11;              // USB bus error
pub const ERR_CONFIGURATION: u8 = 12;        // Configuration error

// Global variables to track the code section and error type
#[used]  // Ensure the linker doesn't optimize these away
#[export_name = "_CODE_SECTION"]
static mut CODE_SECTION: u8 = SECTION_NONE;

#[used]
#[export_name = "_ERROR_FLAG"]
static mut ERROR_FLAG: u8 = ERR_NONE;

/// Set the current code section
///
/// This should be called at the beginning of major code sections
/// to help identify where a panic might occur.
#[inline]
#[no_mangle]
pub extern "C" fn set_code_section(section: u8) {
    unsafe {
        CODE_SECTION = section;
    }
}

/// Set an error flag
///
/// This should be called when an error condition is detected
/// that might lead to a panic.
#[inline]
#[no_mangle]
pub extern "C" fn set_error_flag(error: u8) {
    unsafe {
        ERROR_FLAG = error;
    }
}

/// Clear the error flag
///
/// This should be called when an error condition is resolved.
#[inline]
#[no_mangle]
pub extern "C" fn clear_error_flag() {
    unsafe {
        ERROR_FLAG = ERR_NONE;
    }
}

/// Get the current code section
///
/// This is used by the panic handler to determine where the panic occurred.
#[inline]
#[no_mangle]
pub fn get_code_section() -> u8 {
    unsafe {
        CODE_SECTION
    }
}

/// Get the current error flag
///
/// This is used by the panic handler to determine what error occurred.
#[inline]
#[no_mangle]
pub fn get_error_flag() -> u8 {
    unsafe {
        ERROR_FLAG
    }
}

/// Ultra simple panic handler that just blinks the LED continuously
///
/// This simplified version focuses solely on toggling the LED with minimal code.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Disable interrupts - we're in a panic state
    #[allow(unused_unsafe)]
    unsafe {
        core::arch::asm!("cpsid i");
    }
    
    // Use a pointer-based approach instead of assembly
    unsafe {
        // Try multiple potential GPIO addresses for the LED
        const GPIO1_BASE: u32 = 0x401B8000;
        const GPIO2_BASE: u32 = 0x401BC000;
        const GPIO3_BASE: u32 = 0x401C0000;
        const GPIO4_BASE: u32 = 0x401C4000;
        const GPIO5_BASE: u32 = 0x400C0000;
        const GPIO6_BASE: u32 = 0x42000000;
        const GPIO7_BASE: u32 = 0x42004000;
        const GPIO9_BASE: u32 = 0x4200C000;
        
        // Try different pins - these are common LED pin configurations
        const LED_MASKS: [u32; 5] = [1 << 3, 1 << 5, 1 << 13, 1 << 16, 1 << 0];
        
        let gpio_bases = [
            GPIO1_BASE as *mut u32,
            GPIO2_BASE as *mut u32,
            GPIO3_BASE as *mut u32,
            GPIO4_BASE as *mut u32,
            GPIO5_BASE as *mut u32,
            GPIO6_BASE as *mut u32,
            GPIO7_BASE as *mut u32,
            GPIO9_BASE as *mut u32,
        ];
        
        // Get current code section and error flag to display
        let section = CODE_SECTION;
        let error = ERROR_FLAG;
        
        // Try to configure all GPIOs as output
        for gpio_base in gpio_bases.iter() {
            let gdir = gpio_base.offset(1); // GDIR register is at offset 4 (1 u32)
            let dr = gpio_base.offset(0);   // DR register is at offset 0
            
            for &mask in LED_MASKS.iter() {
                // Configure pins as output
                let current_gdir = core::ptr::read_volatile(gdir);
                core::ptr::write_volatile(gdir, current_gdir | mask);
            }
            
            // DISTINCTIVE PANIC PATTERN
            // The goal is to make it immediately obvious that a panic occurred,
            // and then provide detailed diagnostic information
            
            // First try different masks to find working LED
            for &mask in LED_MASKS.iter() {
                // PANIC INDICATOR - S.O.S. PATTERN (3 short, 3 long, 3 short)
                // This makes it immediately clear we're in a panic state
                
                // 3 short pulses (S)
                for _ in 0..3 {
                    // ON
                    let current_dr = core::ptr::read_volatile(dr);
                    core::ptr::write_volatile(dr, current_dr | mask);
                    
                    // Short pulse (150ms)
                    for _ in 0..500000 {
                        core::hint::spin_loop();
                    }
                    
                    // OFF
                    let current_dr = core::ptr::read_volatile(dr);
                    core::ptr::write_volatile(dr, current_dr & !mask);
                    
                    // Short gap (150ms)
                    for _ in 0..500000 {
                        core::hint::spin_loop();
                    }
                }
                
                // Pause between letters (300ms)
                for _ in 0..1000000 {
                    core::hint::spin_loop();
                }
                
                // 3 long pulses (O)
                for _ in 0..3 {
                    // ON
                    let current_dr = core::ptr::read_volatile(dr);
                    core::ptr::write_volatile(dr, current_dr | mask);
                    
                    // Long pulse (450ms)
                    for _ in 0..1500000 {
                        core::hint::spin_loop();
                    }
                    
                    // OFF
                    let current_dr = core::ptr::read_volatile(dr);
                    core::ptr::write_volatile(dr, current_dr & !mask);
                    
                    // Short gap (150ms)
                    for _ in 0..500000 {
                        core::hint::spin_loop();
                    }
                }
                
                // Pause between letters (300ms)
                for _ in 0..1000000 {
                    core::hint::spin_loop();
                }
                
                // 3 short pulses (S)
                for _ in 0..3 {
                    // ON
                    let current_dr = core::ptr::read_volatile(dr);
                    core::ptr::write_volatile(dr, current_dr | mask);
                    
                    // Short pulse (150ms)
                    for _ in 0..500000 {
                        core::hint::spin_loop();
                    }
                    
                    // OFF
                    let current_dr = core::ptr::read_volatile(dr);
                    core::ptr::write_volatile(dr, current_dr & !mask);
                    
                    // Short gap (150ms)
                    for _ in 0..500000 {
                        core::hint::spin_loop();
                    }
                }
                
                // Long pause before diagnostic info (1 second)
                for _ in 0..3000000 {
                    core::hint::spin_loop();
                }
                
                loop {
                    // DIAGNOSTIC INFORMATION
                    // First indicate SECTION with long blinks (1-12)
                    let display_section = if section > 0 { section } else { 1 };
                    
                    for _ in 0..display_section {
                        // ON
                        let current_dr = core::ptr::read_volatile(dr);
                        core::ptr::write_volatile(dr, current_dr | mask);
                        
                        // Long pulse (750ms)
                        for _ in 0..2500000 {
                            core::hint::spin_loop();
                        }
                        
                        // OFF
                        let current_dr = core::ptr::read_volatile(dr);
                        core::ptr::write_volatile(dr, current_dr & !mask);
                        
                        // Short gap (250ms)
                        for _ in 0..750000 {
                            core::hint::spin_loop();
                        }
                    }
                    
                    // Medium pause between section and error (1.5 second)
                    for _ in 0..5000000 {
                        core::hint::spin_loop();
                    }
                    
                    // Then indicate ERROR with short blinks (1-12)
                    let display_error = if error > 0 { error } else { 1 };
                    
                    for _ in 0..display_error {
                        // ON
                        let current_dr = core::ptr::read_volatile(dr);
                        core::ptr::write_volatile(dr, current_dr | mask);
                        
                        // Short pulse (250ms)
                        for _ in 0..750000 {
                            core::hint::spin_loop();
                        }
                        
                        // OFF
                        let current_dr = core::ptr::read_volatile(dr);
                        core::ptr::write_volatile(dr, current_dr & !mask);
                        
                        // Short gap (250ms)
                        for _ in 0..750000 {
                            core::hint::spin_loop();
                        }
                    }
                    
                    // Very long pause before repeating (3 seconds)
                    for _ in 0..10000000 {
                        core::hint::spin_loop();
                    }
                }
            }
        }
    }
    
    // This will never be reached
    loop {}
}