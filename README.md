# Mumen Controller - Nintendo Switch Pro Controller Firmware

![Mumen Controller Logo](images/mumen-logo.png)

A high-performance, configurable Nintendo Switch Pro Controller firmware for the Teensy 4.0 microcontroller. This firmware offers low-latency input processing, customizable button mapping, and advanced features like SOCD (Simultaneous Opposite Cardinal Direction) handling.

## Table of Contents

- [Features](#features)
- [Hardware Requirements](#hardware-requirements)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Building and Flashing](#building-and-flashing)
- [Configuration](#configuration)
  - [Button Mapping (Pinout)](#button-mapping-pinout)
  - [SOCD Handling](#socd-handling)
  - [Custom Configurations](#custom-configurations)
- [LED Blink Patterns](#led-blink-patterns)
- [Troubleshooting](#troubleshooting)
  - [Common Issues and Solutions](#common-issues-and-solutions)
  - [Debugging Tips](#debugging-tips)
- [Project Structure](#project-structure)
- [Key Components](#key-components)
  - [USB HID Implementation](#usb-hid-implementation)
  - [Input Handling System](#input-handling-system)
  - [Configuration System](#configuration-system)
  - [Panic Handler](#panic-handler)
- [Future Extensions](#future-extensions)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)
- [Contact](#contact)

## Features

- **Low-Latency Input Processing**: Optimized for minimum input lag with 1000Hz polling rate
- **Customizable Button Mapping**: Configure any button to any GPIO pin via TOML files
- **Multiple SOCD Resolution Methods**: Choose between neutral, up-priority, last-win, and more
- **Configurable at Build Time**: Zero runtime overhead for configurations
- **Debouncing**: Hardware and software button debouncing for reliability
- **Analog Stick Calibration**: Support for deadzone, filtering, and calibration
- **Lock Button Feature**: Prevent accidental menu button presses
- **USB HID Compliance**: Proper Nintendo Switch Pro Controller USB descriptors
- **Enhanced USB Stability**: State monitoring with automatic error detection and recovery
- **Status LED Indications**: Visual feedback for different controller states
- **Advanced Debugging**: Comprehensive error reporting with detailed LED blink patterns
- **Power Management**: Efficient power usage for longer battery life on portable setups

## Hardware Requirements

- **Teensy 4.0** microcontroller board
- Digital buttons (tactile switches recommended)
- Analog joysticks (potentiometer-based)
  - Recommended: 10KΩ dual-potentiometer joysticks
- LED for status indication (built-in LED on Teensy 4.0 is used by default)
- Optional lock switch for menu button protection
- Wiring and soldering equipment
- Micro USB cable for programming and data transfer

## Getting Started

### Prerequisites

1. Install Rust and Cargo:
   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install ARM target for Teensy 4.0:
   ```
   rustup target add thumbv7em-none-eabihf
   ```

3. Install cargo-binutils for binary manipulation:
   ```
   cargo install cargo-binutils
   rustup component add llvm-tools-preview
   ```

4. Install Teensy Loader from https://www.pjrc.com/teensy/loader.html

### Building and Flashing

1. Clone this repository:
   ```
   git clone https://github.com/your-username/mumen-controller.git
   cd mumen-controller
   ```

2. Configure your controller by editing the TOML files:
   - `config/pinout/default.toml`: Button-to-pin mappings
   - `config/socd/default.toml`: SOCD resolution methods

3. Build the firmware:
   ```
   cargo build --release
   ```

4. Convert to a format for Teensy Loader:
   ```
   cargo objcopy --release -- -O ihex mumen-controller.hex
   ```

5. Open Teensy Loader, load the hex file, and press the button on Teensy 4.0 to flash

## Configuration

The Mumen Controller firmware uses a compile-time configuration system for maximum performance with zero runtime overhead. All configuration is done through TOML files.

### Button Mapping (Pinout)

Edit `config/pinout/default.toml` to map buttons to Teensy 4.0 pins:

```toml
# Digital inputs
[digital]
button_a = 2       # A button on pin 2
button_b = 3       # B button on pin 3
button_x = 4       # X button on pin 4
button_y = 5       # Y button on pin 5
button_l = 6       # L button on pin 6
button_r = 7       # R button on pin 7
button_zl = 8      # ZL button on pin 8
button_zr = 9      # ZR button on pin 9
button_plus = 10   # Plus button on pin 10
button_minus = 11  # Minus button on pin 11
button_home = 12   # Home button on pin 12
button_capture = 14 # Capture button on pin 14
button_l3 = 15     # Left stick press on pin 15
button_r3 = 16     # Right stick press on pin 16
dpad_up = 17       # D-pad Up on pin 17
dpad_down = 18     # D-pad Down on pin 18
dpad_left = 19     # D-pad Left on pin 19
dpad_right = 20    # D-pad Right on pin 20

# Analog inputs
[analog]
left_stick_x = 23  # Left stick X-axis on pin 23 (A10)
left_stick_y = 22  # Left stick Y-axis on pin 22 (A9)
right_stick_x = 21 # Right stick X-axis on pin 21 (A8)
right_stick_y = 20 # Right stick Y-axis on pin 20 (A7)

# Special functionality
[special]
lock_pin = 33      # Lock functionality on pin 33
```

**Important Notes:**
- Any button or input not listed in the configuration will be automatically disabled
- The pin numbers correspond to the Teensy 4.0 pin numbering scheme
- Analog inputs must be connected to analog-capable pins (check your Teensy 4.0 documentation)
- Digital inputs are read as active-low by default (button press connects pin to ground)

### SOCD Handling

Edit `config/socd/default.toml` to configure how contradictory inputs (like pressing left+right simultaneously) are resolved:

```toml
[resolution_methods]
left_right = "neutral"   # Left + Right = neutral
up_down = "up-priority"  # Up + Down = Up takes priority

[custom_overrides]
# Custom overrides for specific game situations
# "left+down" = "down-left" # Example
```

Available SOCD resolution methods:
- `"neutral"`: Both directions are turned off (Left+Right = center)
- `"last-win"`: The last input pressed takes priority
- `"first-win"`: The first input pressed takes priority
- `"up-priority"`: Up takes priority over down (only for up/down)
- `"second-input-priority"`: Second directional input overrides the first

### Custom Configurations

You can create multiple configuration profiles by adding new TOML files:

1. Create custom configs:
   - `config/pinout/custom.toml`
   - `config/socd/custom.toml`

2. Select which config to use at build time:
   ```
   # Use custom pinout
   PINOUT_CONFIG=custom cargo build --release
   
   # Use custom SOCD configuration
   SOCD_CONFIG=custom cargo build --release
   
   # Use both custom configs
   PINOUT_CONFIG=custom SOCD_CONFIG=custom cargo build --release
   ```

All configuration is processed at compile time, resulting in zero runtime overhead.

## LED Blink Patterns

The firmware uses different LED blink patterns on the Teensy 4.0's onboard LED (pin 13) to indicate various error conditions. These patterns help diagnose issues when troubleshooting:

| Error Type    | Blink Pattern                   | Description                        |
|---------------|--------------------------------|-------------------------------------|
| Hard Fault    | Rapid blinks (5Hz)             | Core ARM processor fault           |
| Memory Error  | Long-short-short               | Memory allocation/pointer error     |
| USB Error     | Long-short-long                | USB communication issue             |
| Init Error    | Continuous ON                  | Hardware initialization failure     |
| Config Error  | Short-long-short               | Invalid configuration               |
| Other/Unknown | SOS pattern (...---...)        | Unspecified error                   |

**Blink Pattern Details:**
- **Long blink**: LED on for 600ms, off for 200ms
- **Short blink**: LED on for 200ms, off for 200ms
- **Rapid blinks**: LED on for 100ms, off for 100ms (5Hz)
- **SOS pattern**: 3 short blinks, 3 long blinks, 3 short blinks, followed by a 1-second pause
- **Continuous ON**: LED remains constantly lit

## Troubleshooting

For comprehensive debugging information, including detailed error codes, diagnostic procedures, and advanced USB troubleshooting, refer to [DEBUG.md](docs/DEBUG.md).

### Common Issues and Solutions

#### Controller not recognized by Nintendo Switch

- **Issue**: The Switch does not detect the controller
- **Solutions**:
  - Ensure the USB cable supports data transfer (some cables are power-only)
  - Verify that the firmware was compiled and flashed correctly
  - Check USB connections and try a different USB port on the dock
  - Try rebooting the Switch

#### Buttons not responding

- **Issue**: Some buttons don't register when pressed
- **Solutions**:
  - Verify the button is properly configured in `config/pinout/default.toml`
  - Check physical connections and wiring
  - Test the pin with a multimeter to ensure it's functioning
  - Use the debugging features described in [DEBUG.md](docs/DEBUG.md) to verify signal detection

#### Erratic or stuck joystick movement

- **Issue**: Joystick input is jumpy or remains in one position
- **Solutions**:
  - Check potentiometer connections and wiring
  - Verify the analog pins are correctly defined in the pinout configuration
  - Try calibrating the joysticks with the Switch's built-in calibration tool
  - Adjust deadzone settings if applicable

#### LED Blink Patterns

- Refer to the [LED Blink Patterns](#led-blink-patterns) section to diagnose specific error conditions
- For detailed explanations of each error pattern, recovery options, and troubleshooting checklists, see [DEBUG.md](docs/DEBUG.md)
- If you see the USB error pattern (long-short-long), see the USB Troubleshooting Checklist in [DEBUG.md](docs/DEBUG.md#usb-panic-troubleshooting-checklist)

#### Lock Button Issues

- **Issue**: Lock button not preventing menu button presses
- **Solutions**:
  - Verify the lock pin is correctly configured in the pinout configuration
  - Check the wiring and physical connection for the lock button
  - Ensure the lock button is properly pulled up/down when not activated

### Debugging Tips

1. **Enable Logging**:
   - Set log level to debug in `Cargo.toml`
   - Connect via serial for additional debugging information
   - Enable USB diagnostics with `cargo build --features="usb-diagnostics"`

2. **Test Individual Components**:
   - Test pins independently using a simple test program
   - Verify USB descriptors are correct
   - Check SOCD handling with deliberate opposing inputs
   - Monitor USB device state transitions to identify connection issues

3. **Hardware Verification**:
   - Use a multimeter to check continuity and proper voltage levels
   - Verify all connections are secure and properly soldered
   - Check for short circuits or cold solder joints
   - Ensure USB cable supports data transfer (not charge-only)

4. **Advanced Debugging**:
   - Refer to [DEBUG.md](docs/DEBUG.md) for in-depth diagnostic procedures
   - Use LED blink patterns to identify specific subsystem failures
   - Check for automatic error recovery attempts (5 rapid blinks)

## Project Structure

- `src/` - Source code
  - `config/` - Configuration system
  - `input/` - Input handling (digital, analog, SOCD, lock)
  - `usb/` - USB HID implementation for Nintendo Switch
  - `panic/` - Panic handler with LED error codes
  - `util/` - Utility functions
- `config/` - Configuration TOML files
  - `pinout/` - Button-to-pin mapping configurations
  - `socd/` - SOCD handling configurations
- `memory.x` - Memory layout for Teensy 4.0
- `build.rs` - Build script for processing configurations

## Key Components

### USB HID Implementation

The firmware implements the Nintendo Switch Pro Controller HID descriptor and communication protocol:

- **Report Format**: Based on the Switch Pro Controller's USB HID descriptor
  - 2 bytes for 16 buttons
  - 1 byte for HAT switch (first nibble) and another nibble
  - 4 bytes for joystick data
  - 1 byte for vendor-specific data
  - 8 bytes for output data

- **USB Communication**:
  - 1000Hz polling rate for minimal latency
  - Proper handling of control requests from the Switch
  - Correct vendor and product IDs for Switch Pro Controller emulation
  - Enhanced state monitoring and error detection
  - Automatic recovery from connection issues

### Input Handling System

The input handling system processes both digital and analog inputs:

- **Digital Input Processing**:
  - Reads GPIO pin states based on pinout configuration
  - Applies debouncing to ensure clean, reliable button inputs
  - Maps physical inputs to logical controller buttons

- **Analog Input Processing**:
  - Reads analog values from designated pins
  - Applies filtering and normalization
  - Maps analog values to joystick ranges expected by the Switch

- **SOCD Handling**:
  - Resolves contradictory inputs (like pressing left+right simultaneously)
  - Configurable resolution methods (neutral, priority, etc.)
  - Applied to both D-pad and analog stick inputs when needed

- **Lock Pin Functionality**:
  - When activated, prevents specific buttons (Home, Plus, Minus, Capture) from being registered
  - Helps avoid accidental menu activation during gameplay
  - Configurable via the pinout configuration

### Configuration System

The configuration system is 100% compile-time with zero runtime overhead:

- **Build Script Processing**:
  - `build.rs` reads TOML configuration files during compilation
  - Generates Rust code with constants based on these configurations
  - Allows selection of different configurations via environment variables

- **Generated Code**:
  - Creates constants for each input's presence and pin mapping
  - Encodes SOCD handling rules as compile-time constants
  - Results in optimized code with no runtime configuration overhead

- **Flexibility**:
  - Multiple configuration profiles can be created
  - Any undefined input is automatically disabled
  - Custom configurations can be selected at build time

### Panic Handler

The panic handler provides visual feedback when errors occur:

- **LED-Based Error Reporting**:
  - Different blink patterns for different error types
  - Helps diagnose issues without additional equipment
  - See [LED Blink Patterns](#led-blink-patterns) for details

- **Error Type Detection**:
  - Automatically infers error type from panic message
  - Categorizes errors into specific types for easier debugging
  - Provides meaningful feedback for common issues

## Future Extensions

The architecture allows for future enhancements:

1. **Advanced Features**:
   - Gyroscope and accelerometer support
   - HD rumble functionality
   - NFC reader for Amiibo compatibility

2. **Multiple Profiles**:
   - Runtime-switchable button mappings
   - Game-specific configurations
   - User-programmable macros

3. **Additional Platforms**:
   - Support for other consoles (PlayStation, Xbox)
   - PC compatibility mode
   - Bluetooth wireless option

4. **Enhanced Customization**:
   - Custom LED patterns for player indicators
   - Advanced button combo programming
   - Adjustable response curves for analog inputs

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add some amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Teensy Project](https://www.pjrc.com/teensy/) for the excellent microcontroller
- [Rust Embedded Community](https://www.rust-lang.org/what/embedded) for tools and libraries
- Nintendo for the Pro Controller design and protocol specifications

## Contact

Your Name - [@your_twitter](https://twitter.com/your_twitter) - email@example.com

Project Link: [https://github.com/your-username/mumen-controller](https://github.com/your-username/mumen-controller)