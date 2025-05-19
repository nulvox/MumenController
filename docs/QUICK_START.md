# Mumen Controller Quick Start Guide

This quick start guide will help you get up and running with your Nintendo Switch Pro Controller firmware for the Teensy 4.0 microcontroller.

## Table of Contents

- [Hardware Requirements](#hardware-requirements)
- [Software Setup](#software-setup)
- [Building and Flashing](#building-and-flashing)
- [Hardware Assembly](#hardware-assembly)
- [Testing Your Controller](#testing-your-controller)
- [Basic Troubleshooting](#basic-troubleshooting)
- [Next Steps](#next-steps)

## Hardware Requirements

Before you begin, ensure you have the following components:

- **Teensy 4.0** microcontroller board
- Digital buttons (tactile switches) for all controller buttons
- Two analog joysticks (10KΩ dual-potentiometer type recommended)
- Optional toggle switch for the lock function
- Micro USB cable (data-capable, not just power)
- Wiring and soldering equipment

## Software Setup

Follow these steps to set up your development environment:

1. **Install Rust and Cargo**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   - Follow the on-screen instructions to complete the installation
   - Restart your terminal after installation

2. **Add the ARM Cortex-M7 Target**
   ```bash
   rustup target add thumbv7em-none-eabihf
   ```

3. **Install Required Tools**
   ```bash
   cargo install cargo-binutils
   rustup component add llvm-tools-preview
   ```

4. **Install Teensy Loader**
   - Download from: https://www.pjrc.com/teensy/loader.html
   - Windows/Mac: Run the installer
   - Linux: Follow the instructions for your distribution

5. **Clone the Repository**
   ```bash
   git clone https://github.com/your-username/mumen-controller.git
   cd mumen-controller
   ```

## Building and Flashing

1. **Review Default Configuration**
   - Check `config/pinout/default.toml` to understand the default pin mapping
   - Review `config/socd/default.toml` for SOCD handling configuration

2. **Build the Firmware**
   ```bash
   cargo build --release
   ```
   - This will compile the firmware with the default configuration
   - The build process reads the TOML configuration files and generates optimized code

3. **Convert for Teensy Loader**
   ```bash
   cargo objcopy --release -- -O ihex mumen-controller.hex
   ```
   - This creates a hex file that the Teensy Loader can flash

4. **Flash to Teensy 4.0**
   - Launch Teensy Loader
   - Click "Open HEX File" and select the generated `mumen-controller.hex`
   - Connect your Teensy 4.0 via USB
   - Press the button on the Teensy to enter programming mode
   - Click "Program" in the Teensy Loader
   - After flashing completes, click "Reboot"

## Hardware Assembly

1. **Plan Your Button Layout**
   - Refer to the pinout configuration to ensure you connect buttons to the correct pins
   - Default digital button connections:
     - A button → Pin 2
     - B button → Pin 3
     - X button → Pin 4
     - Y button → Pin 5
     - And so on (see `config/pinout/default.toml` for complete mapping)

2. **Connect Digital Buttons**
   - Connect one side of each button to its designated Teensy pin
   - Connect the other side of all buttons to GND (ground)
   - Digital inputs are configured as INPUT_PULLUP by default (buttons should connect to ground when pressed)

3. **Connect Analog Joysticks**
   - Connect VCC (3.3V) and GND to the joystick power pins
   - Connect X-axis to the designated analog pin (Pin 23 for left stick X by default)
   - Connect Y-axis to the designated analog pin (Pin 22 for left stick Y by default)
   - Repeat for the right joystick (Pins 21 and 20 by default)

4. **Connect Lock Switch (Optional)**
   - Connect one side of the switch to the lock pin (Pin 33 by default)
   - Connect the other side to GND
   - The lock feature prevents accidental menu button presses when activated

5. **Final Checks**
   - Verify all connections are secure
   - Ensure there are no short circuits
   - Check that the Teensy can be powered via USB

## Testing Your Controller

1. **Initial Power-Up**
   - Connect the Teensy to your PC via USB
   - The onboard LED should light up briefly and then turn off
   - If you see a specific blink pattern, refer to the [LED Blink Patterns](#led-blink-patterns) section in the README.md for troubleshooting

2. **Connect to Nintendo Switch**
   - Connect the Teensy to the Nintendo Switch dock's USB port
   - Navigate to "Controllers" → "Change Grip/Order" on the Switch
   - The Switch should detect a Pro Controller

3. **Test Buttons and Joysticks**
   - Use the Switch's controller test screen:
     - Navigate to System Settings → Controllers and Sensors → Test Input Devices
   - Press each button and move each joystick to verify they're working correctly
   - Test the lock switch (if implemented) by activating it and verifying that the Home, Plus, and Minus buttons are disabled

## Basic Troubleshooting

### LED Blink Patterns

If the Teensy's onboard LED displays a blink pattern, refer to this guide:

- **Rapid blinks (5Hz)**: Hard Fault - Hardware issue or critical firmware error
- **Long-short-short**: Memory Error - RAM or memory allocation issue
- **Long-short-long**: USB Error - Problem with USB communication
- **Continuous ON**: Init Error - Hardware initialization failed
- **Short-long-short**: Config Error - Invalid configuration detected
- **SOS pattern (...---...)**: Other/Unknown error

### Common Issues

1. **Controller not detected by Switch**
   - Ensure USB cable supports data (not just power)
   - Try a different USB port on the dock
   - Verify firmware was correctly flashed

2. **Buttons not responding**
   - Check wiring connections
   - Verify the button is defined in the pinout configuration
   - Test the pin with a multimeter

3. **Erratic joystick movement**
   - Check wiring connections
   - Verify analog pins are correctly defined in the configuration
   - Try calibrating the joysticks in the Switch's controller settings

## Next Steps

After successfully setting up your basic controller, consider these next steps:

1. **Customize Your Configuration**
   - Create custom pinout configurations in `config/pinout/custom.toml`
   - Experiment with different SOCD handling methods in `config/socd/custom.toml`
   - Rebuild with custom configurations:
     ```bash
     PINOUT_CONFIG=custom SOCD_CONFIG=custom cargo build --release
     ```

2. **Enhance Your Build**
   - Add a case or enclosure for your controller
   - Implement additional features using the extension points in the firmware
   - Consider adding labels for buttons and joysticks

3. **Contribute Improvements**
   - Report any issues you encounter
   - Suggest enhancements for the firmware
   - Share your custom configurations with the community

4. **Read the Technical Documentation**
   - For more advanced understanding, read the `TECHNICAL.md` document
   - Learn about the firmware architecture and implementation details

For more detailed information, refer to the main README.md file and TECHNICAL.md documentation.