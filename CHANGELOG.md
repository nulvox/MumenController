# Mumen Controller Firmware Changelog

## Version 1.3.0 (2025-05-18)

### Major Improvements

#### Real USB Implementation

- **Replaced Mock USB Implementation with Real Functionality**:
  - Implemented proper USB bus allocation and device initialization
  - Updated descriptor to exactly match the Nintendo Switch Pro controller
  - Added complete USB communication logic with proper error handling
  - Implemented USB interrupt handling for better responsiveness
  - Optimized memory layout for USB operations

- **USB Device Architecture Improvements**:
  - Implemented thread-safe USB device access with RTIC shared resources
  - Added proper USB report structure implementation for input/output reports
  - Optimized USB polling interval to 1ms for minimum latency
  - Implemented proper vendor and product IDs for Nintendo Switch compatibility
  - Added USB device state tracking and recovery

- **Memory Optimizations**:
  - Added dedicated USB buffer section in memory.x
  - Increased stack size for USB operations
  - Improved memory allocation for USB device
  - Optimized descriptor implementation for memory efficiency

- **Fixed USB Initialization Race Condition**:
  - Resolved a critical race condition between USB initialization sequence and interrupt handler
  - Fixed irregular LED blink patterns that occurred during device startup
  - Implemented proper synchronization mechanisms between initialization process and USB events
  - Improved overall device reliability during the boot sequence
  - Eliminated intermittent connection failures when connecting to hosts
## Version 1.2.0 (2025-05-15)

### Major Improvements

#### Enhanced Error Handling and Diagnostics

- **Fixed LED Error Signaling**: 
  - Standardized LED blink patterns to provide clearer visual feedback for different error types
  - Improved timing consistency in error pattern generation
  - Added more distinguishable patterns between error types
  - Added initial 500ms delay before blinking patterns to make fault states more distinguishable

- **USB Subsystem Improvements**:
  - Added comprehensive USB device state monitoring to detect invalid state transitions
  - Implemented automatic recovery mechanism for USB errors
  - Fixed issue with repeated connect/disconnect cycles causing system panics
  - Added visual feedback during USB recovery attempts

- **Input Configuration Validation**:
  - Added thorough validation of input configurations during initialization
  - Improved error messages for configuration issues
  - Fixed boundary checking for pin assignments
  - Prevented invalid SOCD configurations from causing system failures

- **Initialization Sequence Robustness**:
  - Enhanced initialization stage tracking with clearer visual indicators
  - Added granular error reporting for each initialization stage
  - Implemented initialization stage timeout detection
  - Fixed race condition in peripheral initialization

### Root Cause Analysis: InitError Issue

The `InitError` panic condition was occurring due to a combination of factors:

1. **Race Condition**: The initialization sequence occasionally had timing issues where peripherals were accessed before they were fully initialized, particularly between USB controller setup and input handler initialization.

2. **Unsafe Memory Operations**: Previous memory allocation strategy used unsafe operations that could lead to uninitialized memory access in low-memory conditions.

3. **Unhandled USB Errors**: USB errors during initialization were not properly caught and handled, leading to system panic instead of graceful degradation.

4. **Improper Error Propagation**: Some subsystem initialization errors were not properly propagated to the main error handler, resulting in misleading error signals.

### Diagnostic Improvements

- **Added New Debugging Tools**:
  - Detailed USB state monitoring with logging
  - Initialization stage visual indicators via LED blinks
  - Enhanced error classification for more specific troubleshooting
  - Performance monitoring instrumentation

- **Documentation Updates**:
  - Comprehensive testing procedures in TESTING.md
  - Enhanced debugging guide in DEBUG.md
  - Detailed troubleshooting steps for common errors
  - Improved API documentation with error handling examples

### Other Fixes and Improvements

- Increased heap size from 4KB to 8KB for better stability
- Improved error type inference logic to provide more accurate error classification
- Added timeout detection for peripherals that may hang during initialization
- Enhanced debounce algorithm to prevent spurious inputs during error recovery
- Improved handling of analog input calibration

## Version 1.1.0 (2025-03-10)

- Added support for analog stick calibration
- Implemented multiple SOCD resolution methods
- Added configuration loading from TOML files
- Initial implementation of LED error patterns

## Version 1.0.0 (2025-01-15)

- Initial release of Mumen Controller firmware
- Basic Nintendo Switch Pro controller emulation
- Digital and analog input support
- USB HID implementation