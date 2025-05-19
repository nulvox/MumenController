# Mumen Controller Testing Procedures

This document outlines the testing procedures for the Mumen Controller firmware, focusing on validating error signaling, USB device state monitoring, and input configuration systems.

## Table of Contents

- [LED Error Signaling Tests](#led-error-signaling-tests)
- [USB Device State Tests](#usb-device-state-tests)
- [Input Configuration Tests](#input-configuration-tests)
- [Diagnostic Instrumentation](#diagnostic-instrumentation)
- [Integration Testing](#integration-testing)

## LED Error Signaling Tests

### Test 1: LED Error Pattern Verification

**Objective**: Verify that each error type produces the correct LED blinking pattern.

**Procedure**:

1. Create a test harness that triggers each error type:
   ```bash
   cargo test --test led_patterns
   ```

2. For each error type (HardFault, MemoryError, UsbError, InitError, ConfigError, Other):
   - Manually trigger the error
   - Observe the LED blinking pattern
   - Verify it matches the documented pattern in DEBUG.md
   - Confirm the 500ms initial delay is present before each pattern begins

**Expected Results**:

| Error Type | Expected Pattern | Initial Delay | Pass/Fail |
|------------|------------------|---------------|-----------|
| HardFault | 3 short blinks at 5Hz | 500ms | - |
| MemoryError | 1 long, 2 short blinks | 500ms | - |
| UsbError | 2 long, 1 short blinks | 500ms | - |
| InitError | 3 long blinks | 500ms | - |
| ConfigError | 4 long blinks | 500ms | - |
| Other | SOS pattern | 500ms | - |

### Test 2: Initialization Stage Indicators

**Objective**: Verify that initialization stage indicators blink correctly.

**Procedure**:

1. Enable verbose debug mode in the firmware:
   ```rust
   // Add to main.rs before initialization sequence
   #[cfg(feature = "debug")]
   let debug_init = true;
   ```

2. Power on the device and observe the LED blink sequence during startup.

3. Verify each stage (1-8) is indicated by the corresponding number of blinks.

**Expected Results**:
- Stage 1 (Core System): 1 blink
- Stage 2 (Peripheral Setup): 2 blinks
- Stage 3 (LED Initialization): 3 blinks
- Stage 4 (Input Handler Setup): 4 blinks
- Stage 5 (USB Subsystem): 5 blinks
- Stage 6 (Configuration Loading): 6 blinks
- Stage 7 (Controller State): 7 blinks
- Stage 8 (Main Loop Ready): 8 blinks

## USB Device State Tests

### Test 1: USB Connection State Monitoring

**Objective**: Verify that the firmware correctly detects and handles USB connection states.

**Procedure**:

1. Connect the controller to a Nintendo Switch.

2. Monitor the log output via debug port (if available) or observe the LED activity:
   ```bash
   cargo run --example usb_monitor
   ```

3. Disconnect and reconnect the USB cable several times.

4. Verify that the firmware:
   - Detects the disconnection (LED off or specific pattern)
   - Re-establishes connection when reconnected (LED resumes normal operation)

**Expected Results**:
- USB state transitions are detected correctly
- No error LEDs triggered during normal connect/disconnect cycles
- Device recovers automatically when reconnected

### Test 2: USB Error Recovery

**Objective**: Verify that the firmware recovers from USB errors.

**Procedure**:

1. Connect the controller to a Switch while running the stress test firmware:
   ```bash
   cargo run --example usb_stress_test
   ```

2. Simulate USB errors by one of these methods:
   - Use a poor-quality or intermittent USB cable
   - Generate USB bus noise using a USB protocol analyzer
   - Use the built-in error simulation: Press B+ZR+Home simultaneously

3. Observe the recovery behavior.

**Expected Results**:
- After detecting multiple USB errors (>10), the device should attempt a reset
- LED should flash 5 times rapidly during reset
- After reset, normal operation should resume
- The device should not enter a panic state unless errors persist

## Input Configuration Tests

### Test 1: Input Configuration Validation

**Objective**: Verify that the input configuration system validates settings properly.

**Procedure**:

1. Create a test configuration with intentional errors:
   ```toml
   # Test with invalid pinout
   digital_pin_b = 255  # Invalid pin number
   ```

2. Flash the firmware with the invalid configuration.

3. Observe startup behavior and check for ConfigError pattern.

**Expected Results**:
- Firmware should detect invalid configuration
- LED should show ConfigError pattern (4 long blinks)
- Debug output should indicate the specific configuration problem

### Test 2: Input Debounce and SOCD Verification

**Objective**: Verify that input debouncing and SOCD handling work correctly.

**Procedure**:

1. Connect input testing apparatus or use physical buttons.

2. For debounce testing:
   - Simulate button bounce by rapidly pressing/releasing or using a test jig
   - Verify that only a single input is registered

3. For SOCD testing:
   - Press opposing directions simultaneously (e.g., Left+Right or Up+Down)
   - Verify that the configured SOCD method (Neutral, Last-Win, etc.) is applied

**Expected Results**:
- Debounced inputs should not register false positives during noisy inputs
- SOCD resolution should follow the configured method consistently

## Diagnostic Instrumentation

### Test 1: Debug Mode Instrumentation

**Objective**: Verify that the diagnostic instrumentation provides useful information.

**Procedure**:

1. Enable debug instrumentation:
   ```bash
   cargo build --features="debug-instrumentation"
   ```

2. Connect the controller to the debugging port.

3. Run the controller through a sequence of operations:
   - Initialization
   - Button presses
   - Analog stick movements
   - Error conditions

4. Monitor and capture the debug output.

**Expected Results**:
- Debug output should show each initialization stage
- Input events should be logged with timing information
- USB state transitions should be recorded
- Memory usage metrics should be available
- When errors occur, detailed diagnostic information should be provided

### Test 2: Performance Monitoring

**Objective**: Verify that the firmware maintains timing requirements under load.

**Procedure**:

1. Build firmware with timing diagnostics:
   ```bash
   cargo build --features="timing-diagnostics"
   ```

2. Run a performance test suite:
   ```bash
   cargo run --example performance_test
   ```

3. Monitor timing metrics during heavy input activity.

**Expected Results**:
- Input-to-report latency should remain below 2ms in all cases
- USB polling should meet the designated intervals (8ms)
- No timing violations should be reported during normal operation

## Integration Testing

### Test 1: Full System Test with Nintendo Switch

**Objective**: Verify that all systems work together correctly with actual hardware.

**Procedure**:

1. Flash the controller with the latest firmware.

2. Connect to Nintendo Switch and navigate to the controller calibration screen.

3. Test all inputs:
   - All digital buttons
   - Analog stick movements (full range)
   - Special button combinations

4. Play a game that uses multiple controller features (e.g., Super Smash Bros).

**Expected Results**:
- All inputs should be recognized correctly
- No spurious inputs should be detected
- Analog stick movement should be smooth and accurate
- No disconnections or errors should occur during extended play

### Test 2: Recovery from Common Error Conditions

**Objective**: Verify that the controller can recover from common error conditions.

**Procedure**:

1. Test various error scenarios:
   - Disconnect/reconnect USB during operation
   - Rapid button mashing across multiple inputs
   - Hold multiple buttons while connecting/disconnecting
   - Connect to different USB ports/devices

2. Observe recovery behavior after each scenario.

**Expected Results**:
- Controller should recover from all common error conditions without requiring reset
- LED error patterns should correctly indicate any unrecoverable issues
- After recoverable errors, full functionality should be restored