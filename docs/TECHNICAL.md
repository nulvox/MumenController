# Mumen Controller Technical Documentation

This document provides detailed technical information about the key components of the Nintendo Switch Pro Controller firmware for the Teensy 4.0 microcontroller.

## Table of Contents

- [System Architecture](#system-architecture)
- [USB HID Implementation](#usb-hid-implementation)
- [Input Handling System](#input-handling-system)
- [Configuration System](#configuration-system)
- [Panic Handler and Error Reporting](#panic-handler-and-error-reporting)
- [Performance Considerations](#performance-considerations)
- [Technical Implementation Notes](#technical-implementation-notes)

## System Architecture

The Mumen Controller firmware is built on the RTIC (Real-Time Interrupt-driven Concurrency) framework to ensure real-time performance and reliable response times. The system architecture consists of the following key components:

```
Main
├── USB HID Implementation
│   ├── HID Descriptor
│   ├── USB Communication
│   └── Report Generation
├── Button/Input Handling
│   ├── Digital Input
│   ├── Analog Input
│   ├── Input Debouncing
│   ├── SOCD Handling
│   └── Lock Logic
├── Panic Handler
│   ├── LED Error Codes
│   └── USB Error Logging
└── Configuration System
    ├── Pinout Config
    ├── SOCD Config
    └── Build Features
```

### Real-Time Processing

The firmware uses RTIC to prioritize tasks and ensure deterministic execution:

- **Input Processing Task**: Handles button reads and processing (high priority)
- **USB Communication Task**: Manages USB communication (high priority)
- **LED Status Task**: Updates status LED indicators (low priority)

This task prioritization ensures that input handling and USB communication receive the highest processing priority, minimizing input lag.

## USB HID Implementation

The USB HID implementation is based on the Nintendo Switch Pro Controller USB protocol and uses a real USB device implementation with the proper descriptor and communication protocols.

### HID Descriptor

The firmware implements the Nintendo Switch Pro Controller HID descriptor based on the USB HID specification:

```rust
pub struct SwitchProReport {
    /// 16 buttons (A, B, X, Y, etc.)
    pub buttons: [bool; 16],
    /// HAT/D-pad direction (0-7, 8 = released)
    pub hat: u8,
    /// Left stick X coordinate
    pub left_stick_x: u8,
    /// Left stick Y coordinate
    pub left_stick_y: u8,
    /// Right stick X coordinate
    pub right_stick_x: u8,
    /// Right stick Y coordinate
    pub right_stick_y: u8,
    /// Vendor specific data
    pub vendor_spec: u8,
}
```

#### Button Mapping in the HID Report

The 16 buttons are mapped in the first two bytes of the report as follows:

| Byte | Bit | Button        |
|------|-----|---------------|
| 0    | 0   | Y Button      |
| 0    | 1   | X Button      |
| 0    | 2   | B Button      |
| 0    | 3   | A Button      |
| 0    | 4   | R Button      |
| 0    | 5   | ZR Button     |
| 0    | 6   | Minus Button  |
| 0    | 7   | Plus Button   |
| 1    | 0   | Left Stick    |
| 1    | 1   | Right Stick   |
| 1    | 2   | Home Button   |
| 1    | 3   | Capture Button|
| 1    | 4   | L Button      |
| 1    | 5   | ZL Button     |
| 1    | 6   | (Unused)      |
| 1    | 7   | (Unused)      |

### HAT Switch Implementation

The D-pad is implemented as a HAT switch in the third byte of the report:

| Value | Direction        |
|-------|------------------|
| 0     | N (Up)           |
| 1     | NE (Up+Right)    |
| 2     | E (Right)        |
| 3     | SE (Down+Right)  |
| 4     | S (Down)         |
| 5     | SW (Down+Left)   |
| 6     | W (Left)         |
| 7     | NW (Up+Left)     |
| 8     | Released (Neutral)|

The HAT value is stored in the first 4 bits of the third byte of the USB report. If the HAT is released (value 8), it's represented as 0x0F in the actual USB report.

### Analog Sticks Format

Analog sticks use 8-bit values per axis, with a range of 0-255:

- 128 represents the center position
- 0 represents the minimum position (left/up)
- 255 represents the maximum position (right/down)

## Input Handling System

The input handling system is responsible for reading hardware inputs and translating them into controller states.

### Digital Input Processing

Digital inputs are processed through the `DigitalInputProcessor` which:

1. Reads GPIO pin states based on pinout configuration
2. Applies debouncing to filter out noise and switch bounce
3. Maps physical inputs to logical controller buttons
4. Handles special cases like the lock feature

```rust
// Simplified example of digital input processing
pub fn process_digital_inputs(pins: &[u8], debounce_ms: u32) -> ButtonState {
    let mut state = ButtonState::default();
    
    for (button_idx, &pin) in pins.iter().enumerate() {
        if pin == 0 {
            continue; // Skip unconfigured buttons
        }
        
        let raw_state = read_gpio_pin(pin);
        let debounced_state = apply_debouncing(raw_state, debounce_ms);
        
        if debounced_state {
            state.set_button(button_idx, true);
        }
    }
    
    state
}
```

### Analog Input Processing

Analog inputs are processed through the `AnalogInputProcessor` which:

1. Reads ADC values from designated pins
2. Applies filtering to smooth out noise
3. Maps raw ADC values to normalized joystick ranges
4. Handles calibration and deadzones

```rust
// Simplified example of analog input processing
pub fn process_analog_inputs(
    analog_pins: &[u8], 
    deadzone: u16,
    calibration: &Calibration
) -> JoystickState {
    let mut state = JoystickState::default();
    
    // Process left stick
    if analog_pins[0] != 0 && analog_pins[1] != 0 {
        let raw_x = read_adc(analog_pins[0]);
        let raw_y = read_adc(analog_pins[1]);
        
        let (x, y) = apply_calibration_and_deadzone(
            raw_x, raw_y, deadzone, &calibration
        );
        
        state.left_stick = (x, y);
    }
    
    // Process right stick (similar to left)
    // ...
    
    state
}
```

### SOCD Handling Implementation

SOCD (Simultaneous Opposite Cardinal Direction) handling resolves situations where opposite directions are pressed simultaneously. The implementation supports various resolution methods:

- **Neutral**: Both directions are turned off (e.g., LEFT+RIGHT = neutral)
- **LastWin**: The last input pressed takes priority
- **FirstWin**: The first input pressed takes priority 
- **UpPriority**: Up takes priority over down (for up/down only)
- **SecondInputPriority**: Second directional input overrides the first

```rust
// Example of SOCD resolution for left and right inputs
fn resolve_left_right(&self, left: bool, right: bool) -> (bool, bool) {
    if !left || !right {
        // No conflict
        return (left, right);
    }
    
    match self.left_right_method {
        SocdMethod::Neutral => (false, false),
        SocdMethod::LastWin => {
            if self.last_left_time > self.last_right_time {
                (true, false)
            } else {
                (false, true)
            }
        },
        SocdMethod::FirstWin => {
            if self.first_left_time < self.first_right_time {
                (true, false)
            } else {
                (false, true)
            }
        },
        // Other methods...
    }
}
```

### Lock Pin Functionality

The lock pin functionality prevents accidental presses of menu buttons (Home, Plus, Minus, Capture) when activated. This is particularly useful during gameplay to avoid accidentally accessing system menus.

When the lock pin is active:
1. The firmware detects the lock pin state at the beginning of each input processing cycle
2. If active, the firmware masks out the specified menu button inputs
3. All other inputs are processed normally

```rust
// Simplified lock functionality implementation
pub fn process(&self, button_states: &[bool; 14]) -> [bool; 14] {
    let mut result = *button_states;
    
    if self.is_locked {
        // Mask out menu buttons when locked
        for &button in &self.locked_buttons {
            let idx = button.to_index();
            result[idx] = false;
        }
    }
    
    result
}
```

## Configuration System

The configuration system provides a flexible, 100% compile-time configuration approach with zero runtime overhead.

### Build-Time Configuration Processing

The `build.rs` script processes TOML configuration files at compile time:

1. Reads configuration files based on environment variables or defaults
2. Deserializes TOML into Rust structures
3. Generates Rust code with constants and type-safe enums
4. The generated code is included in the compiled firmware

```rust
// Example of generated configuration code (simplified)
// This is generated by the build script

// Pinout configuration
pub const HAS_BUTTON_A: bool = true;
pub const BUTTON_A_PIN: u8 = 2;
pub const HAS_BUTTON_B: bool = true;
pub const BUTTON_B_PIN: u8 = 3;
// ... other buttons

// SOCD configuration
pub enum SocdMode {
    Neutral,
    Priority,
    LastInput,
}

pub const DPAD_UPDOWN_MODE: SocdMode = SocdMode::Neutral;
pub const DPAD_LEFTRIGHT_MODE: SocdMode = SocdMode::Neutral;
```

### Configuration File Structure

#### Pinout Configuration

The pinout configuration maps buttons and inputs to specific GPIO pins on the Teensy 4.0:

```toml
# Digital inputs
[digital]
button_a = 2
button_b = 3
# ... other buttons

# Analog inputs
[analog]
left_stick_x = 23
left_stick_y = 22
# ... other analog inputs

# Special functionality
[special]
lock_pin = 33
```

#### SOCD Configuration

The SOCD configuration defines how contradictory inputs are resolved:

```toml
[resolution_methods]
left_right = "neutral"
up_down = "up-priority"

[custom_overrides]
# Custom resolution patterns (if needed)
```

### Dynamic Configuration Selection

The firmware supports selecting different configuration profiles by setting environment variables during the build process:

```bash
# Use custom pinout
PINOUT_CONFIG=custom cargo build --release

# Use custom SOCD configuration
SOCD_CONFIG=custom cargo build --release
```

## Panic Handler and Error Reporting

The panic handler provides visual feedback when errors occur, using the onboard LED to display different blink patterns.

### Error Type Classification

Errors are classified into specific types for easier diagnosis:

```rust
pub enum ErrorType {
    HardFault,    // Core ARM processor fault
    MemoryError,  // Memory allocation/pointer issues
    UsbError,     // USB communication problems
    InitError,    // Hardware initialization failure
    ConfigError,  // Invalid configuration
    Other,        // Other unspecified errors
}
```

### LED Blink Patterns

Each error type has a unique LED blink pattern:

| Error Type    | Blink Pattern                   | Description                        |
|---------------|--------------------------------|-------------------------------------|
| HardFault     | Rapid blinks (5Hz)             | Core ARM processor fault           |
| MemoryError   | Long-short-short               | Memory allocation/pointer error     |
| UsbError      | Long-short-long                | USB communication issue             |
| InitError     | Continuous ON                  | Hardware initialization failure     |
| ConfigError   | Short-long-short               | Invalid configuration               |
| Other/Unknown | SOS pattern (...---...)        | Unspecified error                   |

Implementation details:
- **Long blink**: LED on for 600ms, off for 200ms
- **Short blink**: LED on for 200ms, off for 200ms
- **Rapid blinks**: LED on for 100ms, off for 100ms (5Hz)
- **SOS pattern**: 3 short blinks, 3 long blinks, 3 short blinks, followed by a 1-second pause

```rust
// Example: Pattern for Memory Error (Long-short-short)
fn blink_pattern_memory_error(&mut self) -> ! {
    loop {
        self.blink_long();
        self.blink_short();
        self.blink_short();
        self.delay_ms(1000); // Pause between pattern repetitions
    }
}
```

### Error Type Inference

The panic handler attempts to infer the error type from the panic message:

```rust
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
```

## Performance Considerations

The firmware is optimized for low latency and reliable performance:

### Latency Optimization Techniques

1. **100% Compile-Time Configuration**: All configuration decisions resolved at compile time with zero runtime overhead
2. **Prioritized Tasks**: Using RTIC's priority system to ensure input processing and USB report generation have higher priority than non-critical tasks
3. **Efficient Memory Usage**: Static memory allocation only, no dynamic allocation
4. **Interrupt-Driven Design**: Using interrupts for GPIO changes to detect button presses immediately
5. **Optimized USB Polling**: Configured for 1000Hz (1ms) polling intervals
6. **Prebuffering**: Report structures prepared in advance to minimize processing when needed
7. **Direct Register Access**: Where appropriate, direct register access for time-critical operations

### Memory Optimization

The firmware is optimized to minimize memory usage:

1. **Static Allocation**: No heap usage to avoid memory fragmentation and allocation overhead
2. **Compile-Time Constants**: Configuration values are embedded as constants
3. **Efficient Data Structures**: Compact, bitpacked structures for button states
4. **Stack Usage Control**: Careful control of stack depth to avoid overflows

## Technical Implementation Notes

### Debouncing Algorithm

The firmware implements a time-based debouncing algorithm to handle mechanical button bounce:

```rust
// Simplified debouncing algorithm
pub fn debounce(&mut self, current_state: bool, current_time_ms: u32) -> bool {
    if current_state != self.last_state {
        self.last_transition_time = current_time_ms;
        self.last_state = current_state;
        return self.stable_state;
    }
    
    if current_time_ms - self.last_transition_time >= self.debounce_time_ms {
        self.stable_state = current_state;
    }
    
    self.stable_state
}
```

### USB Implementation Notes

The USB implementation uses the `usbd-hid` and `usb-device` crates to implement the USB HID protocol. The firmware:

1. Defines a custom HID report descriptor matching the Switch Pro Controller based on the standard descriptor
2. Creates a real USB device implementation with proper Nintendo VID (0x057E) and PID (0x2009)
3. Implements the required USB control requests and endpoint configurations
4. Manages the USB polling interval for 1ms updates to ensure low-latency response
5. Tracks connection state and handles unexpected disconnections
6. Includes built-in error recovery mechanisms

The USB device initialization configures the appropriate product strings, power requirements (500mA), and maximum packet sizes for optimal performance.

### USB Error Handling and Recovery

The firmware implements a robust error-handling system for USB communication:

1. **State Transition Monitoring**: The firmware tracks USB device state transitions (Default → Configured → Default) to detect connection and disconnection events.

2. **Error Detection**:
   - Unexpected disconnections are detected and logged
   - State transition errors are counted and monitored
   - Communication errors with the host are tracked

3. **Automatic Recovery**:
   - After multiple state transition errors (>5), the device will automatically attempt recovery
   - Software reset of the USB device state is performed
   - Buffers are cleared and the connection state is reset

4. **Graceful Degradation**:
   - The firmware can continue operating in a degraded mode during temporary USB issues
   - It will attempt to re-establish connection on subsequent polling cycles

5. **Logging and Diagnostics**:
   - State transitions are logged for debugging purposes
   - Error conditions are reported through the LED error reporting system
   - USB connection status is tracked globally for system-wide awareness

### Analog Joystick Processing

The analog joysticks use 10-bit ADC readings that are processed and normalized:

1. Raw ADC readings (0-1023) are acquired
2. Deadzone is applied to filter out small movements near center
3. Values are normalized to the 0-255 range expected by the Switch
4. Calibration adjustments are applied based on neutral position

### Build System Integration

The build system uses Cargo build scripts to:

1. Read configuration files
2. Generate constant definitions
3. Allow configuration selection via environment variables
4. Enable conditional compilation of features
5. Properly handle dependencies and rebuilding when configurations change