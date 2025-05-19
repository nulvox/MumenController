# Mumen Controller Firmware Debugging Guide

This guide documents the debugging capabilities of the Mumen Controller firmware, including LED debug patterns, initialization stages, and troubleshooting techniques.

## LED Debug Patterns

### Error Type Patterns

The onboard LED (pin 13) is used to signal different types of errors through distinct blinking patterns. When a panic occurs, the LED will begin with a 500ms delay (LED off) to clearly mark the start of an error pattern, then continuously repeat the specific pattern that indicates the type of error.

| Error Type | Blink Pattern | Description |
|------------|---------------|-------------|
| HardFault | Continuous short blinks (equal on/off) | CPU detected a fault condition (illegal memory access, etc.) |
| MemoryError | Long-short-short | Memory allocation or access failure |
| UsbError | Long-short-long | USB initialization or communication failure |
| InitError | 3 long blinks | Peripheral or subsystem initialization error |
| ConfigError | Short-long-short | Configuration error (missing/invalid config) |
| Other | SOS pattern (3 short, 3 long, 3 short) | Unclassified error |

Each short blink is 200ms on, 200ms off. Each long blink is 600ms on, 200ms off. All patterns begin with a 500ms delay to help distinguish the start of a new pattern cycle, followed by a 1000ms pause between pattern repetitions, making error identification more reliable.

### Initialization Stage Indicators

During startup, the firmware uses LED blinks to indicate which initialization stage is being executed. The `debug_blink_stage` function produces a specific number of blinks corresponding to the stage number, helping identify where initialization might be failing:

| Blinks | Stage | Description |
|--------|-------|-------------|
| 1 | Core System | Memory and critical system components |
| 2 | Peripheral Setup | GPIO, ADC setup, timer configuration |
| 3 | LED Initialization | Onboard LED setup (debug channel established) |
| 4 | Input Handler Setup | Digital/analog inputs, SOCD, lock handlers |
| 5 | USB Subsystem | USB device initialization |
| 6 | Configuration Loading | Loading configuration from storage |
| 7 | Controller State | Setting up initial controller state |
| 8 | Main Loop Ready | System fully initialized, entering main loop |

Each stage blink consists of the LED turning on for 100ms and off for 100ms, repeated the number of times corresponding to the stage number, followed by a 500ms pause.

If initialization fails, the system will transition to the error pattern corresponding to the failure type (such as the USB error pattern if the USB initialization fails at stage 5).

## Progressive Initialization

The firmware uses a staged initialization approach to ensure that each component is properly initialized before proceeding to dependent components. This helps isolate issues and ensures that the LED debugging system is available as early as possible.

#### Stage 1: Core System Initialization

#### Stage 2: Peripheral Setup

#### Stage 3: LED Initialization

#### Stage 4: Input Handler Setup

#### Stage 5: USB Subsystem

#### Stage 6: Configuration Loading

#### Stage 7: Controller State Setup

#### Stage 8: Main Loop Ready

## Using Debug Instrumentation

The firmware provides a `debug_blink_stage()` function that can be used to insert debug points in your code. This function causes the LED to blink a specified number of times, allowing you to track execution flow.

### Adding Debug Points

To add debug instrumentation to your code:

```rust
// Import the debug_blink_stage function if not in scope
use crate::debug::debug_blink_stage;

// Later in your code
fn my_function() {
    // Signal entering this function with 2 blinks
    debug_blink_stage(2);
    
    // Perform some operations
    // ...
    
    // Signal a specific point in the function with 3 blinks
    debug_blink_stage(3);
    
    // More operations
    // ...
}
```

### Tracking Execution Flow

You can use different numbers of blinks to track the execution path through your code:

```rust
fn complex_function() {
    debug_blink_stage(1); // Starting the function
    
    if condition_a {
        debug_blink_stage(2); // Path A
        // ...
    } else if condition_b {
        debug_blink_stage(3); // Path B
        // ...
    } else {
        debug_blink_stage(4); // Default path
        // ...
    }
    
    debug_blink_stage(5); // End of function
}
```

## Troubleshooting Common Issues

### Memory Errors

If you encounter a memory error (1 long, 2 short blinks):

1. Check the heap size configuration in `main.rs`
2. Verify that you're not exceeding stack limits with large local variables
3. Look for potential memory leaks or double-free issues
4. Review the memory layout in `memory.x` to ensure proper allocation of RAM regions

### USB Errors

If you encounter a USB error (long-short-long blink pattern):

1. Check USB physical connections (cable, ports)
2. Verify USB descriptor configuration
3. Check for conflicts with other USB devices
4. Verify USB controller initialization parameters
5. Check if USB device state transitions are valid
6. Monitor USB error count in debug logs

Common USB initialization issues include:
- Incorrect USB descriptor configuration
- Insufficient power from the USB port
- Interference from other devices on the same USB controller
- Timing issues during the USB enumeration process
- Host-side driver compatibility problems

The firmware includes enhanced USB device state monitoring with automatic error recovery:

* The USB device state is continuously monitored during polling
* Invalid state transitions are detected and logged
* If too many errors occur in succession (>10), the device will automatically attempt to reset itself
* During reset, the LED will blink 5 times rapidly to indicate recovery is in progress

### Initialization Errors

If you encounter an initialization error (3 long blinks pattern):

1. Identify which stage failed using the initialization stage indicators (by counting the blinks before failure)
2. Check for missing dependencies or incorrect initialization order
3. Verify that all required hardware is properly connected
4. Check that pin configurations match your actual hardware setup
5. Check debug logs for detailed error messages about the specific initialization failure

Common initialization failures:

| Stage | Common Issues | Troubleshooting |
|-------|---------------|-----------------|
| 1-2 | Core hardware issues | Check power supply, clock configuration |
| 3 | LED pin configuration | Verify LED pin mapping |
| 4 | Input hardware connectivity | Check input pin configurations, wiring |
| 5 | USB configuration/connectivity | Check USB descriptors, connection, power |
| 6 | Configuration loading | Verify config files, check storage integrity |
| 7-8 | State initialization | Check component dependencies, initialization order |

The initialization error handling provides:

* Clear identification of which initialization stage failed via blink patterns
* More detailed error classification in logs
* Specific error codes for different types of initialization failures
* Automatic recovery attempts for non-critical initialization errors

### Configuration Errors

If you encounter a configuration error (short-long-short blink pattern):

1. Check that all required configuration files exist
2. Verify that configuration files are properly formatted
3. Ensure configuration values are within valid ranges
4. Look for conflicting configuration settings
5. Check for hardware/configuration mismatches

Common configuration issues:

* Missing or corrupted configuration files
* Invalid pin mappings (pins assigned to multiple functions)
* Out-of-range analog calibration values
* Invalid SOCD (Simultaneous Opposite Cardinal Direction) settings
* Incompatible button mapping configurations

The configuration subsystem verifies all settings during startup and will report specific errors for invalid configurations.

If you encounter a configuration error (4 long blinks):

1. Verify that configuration files exist and are properly formatted
2. Check that configuration values are within valid ranges
3. Look for potential format issues or encoding problems in configuration files
4. Verify that required configuration parameters are present

### Hardware Considerations

- Verify that the Teensy 4.0 board is properly powered (stable 5V supply)
- Check for proper connection of all input pins
- Ensure that any external components are correctly wired
- Consider environmental factors like EMI, temperature, and vibration that might affect operation
- Check that the Teensy's USB connection is stable and not affected by signal integrity issues

## Memory Optimizations

The Mumen Controller firmware includes several memory optimizations to prevent runtime memory errors, particularly during extended operation. These optimizations help maintain system stability while allowing all functionality to operate efficiently.

### Implemented Memory Optimizations

#### Fixed-Size Arrays Instead of Dynamic Vectors

Dynamic memory allocation (such as using `Vec<T>`) can cause heap fragmentation over time in embedded systems. The firmware uses fixed-size arrays with counters instead:

```rust
// Instead of:
struct LockHandler {
    locked_buttons: Vec<LockableButton>,
    // ...other fields
}

// Optimized version:
const MAX_LOCKABLE_BUTTONS: usize = 4;
struct LockHandler {
    locked_buttons: [Option<LockableButton>; MAX_LOCKABLE_BUTTONS],
    button_count: usize,
    // ...other fields
}
```

This approach:
- Prevents heap fragmentation
- Makes memory usage predictable
- Reduces allocation overhead
- Eliminates potential allocation failures

#### Increased Heap Size

The firmware allocates 8KB for the heap (increased from the original 4KB):

```rust
static mut HEAP: MaybeUninit<[u8; 8192]> = MaybeUninit::uninit();
unsafe {
    let heap_ptr = HEAP.as_mut_ptr() as *mut u8;
    ALLOCATOR.lock().init(heap_ptr, 8192);
}
```

The larger heap provides sufficient headroom for runtime operations and helps prevent memory exhaustion during extended use.

### Signs of Memory-Related Issues

Memory errors typically manifest with these symptoms:

1. **LED Error Pattern**: Memory errors show as 1 long blink followed by 2 short blinks in a repeating pattern
2. **Progressive Degradation**:
   - Erratic behavior after extended runtime (hours of operation)
   - Initially stable operation, followed by occasional glitches
   - Increasing frequency of glitches until failure occurs
3. **Timing-Related**:
   - Issues more likely during rapid input sequences
   - Problems may appear when multiple features are used simultaneously
   - Memory errors might occur during or shortly after configuration changes

If you observe these symptoms, particularly after making firmware modifications, you should investigate potential memory issues.

### Additional Troubleshooting for Memory Issues

#### Adjusting Heap Size

If you need to further adjust the heap size:

1. Locate the heap initialization in `main.rs`:

```rust
static mut HEAP: MaybeUninit<[u8; 8192]> = MaybeUninit::uninit();
unsafe {
    let heap_ptr = HEAP.as_mut_ptr() as *mut u8;
    ALLOCATOR.lock().init(heap_ptr, 8192);
}
```

2. Modify both the array size and the initialization parameter to the desired size (e.g., 12KB):

```rust
static mut HEAP: MaybeUninit<[u8; 12288]> = MaybeUninit::uninit();
unsafe {
    let heap_ptr = HEAP.as_mut_ptr() as *mut u8;
    ALLOCATOR.lock().init(heap_ptr, 12288);
}
```

Note that increasing the heap size reduces available memory for other purposes, so balance this carefully against your system's total available RAM.

#### Best Practices for Embedded Memory Management

When modifying or extending the firmware:

1. **Avoid Dynamic Allocation**:
   - Prefer fixed-size arrays over `Vec<T>` or similar collections
   - Use stack-allocated buffers when possible
   - Consider using static allocation for long-lived objects

2. **Manage Buffer Sizes**:
   - Ensure fixed buffer sizes are adequate for their purpose
   - Add checks to prevent buffer overflows
   - Consider fallback behavior for when data exceeds buffer capacity

3. **Avoid Recursive Algorithms**:
   - Recursive functions can cause stack overflow
   - Rewrite recursive algorithms as iterative ones
   - If recursion is necessary, add depth limits

#### Memory Profiling

To profile memory usage:

1. **Add Runtime Reporting**:
   
   ```rust
   // This requires the allocator to expose stats
   pub fn log_memory_usage() {
       let stats = ALLOCATOR.lock().stats();
       log::info!(
           "Memory usage: {}/{} bytes ({:.1}%)",
           stats.used,
           stats.total,
           (stats.used as f32 / stats.total as f32) * 100.0
       );
   }
   ```

2. **Create Memory Checkpoints**:
   
   Place memory usage reporting at key points in your code to identify which operations consume memory.

3. **Monitor Fragmentation**:
   
   Track the largest available contiguous memory block to detect fragmentation:
   
   ```rust
   log::info!("Largest free block: {} bytes", ALLOCATOR.lock().largest_free_block());
   ```

By following these practices, you can maintain the memory stability of the firmware even when adding new features or making modifications.

## Advanced Debugging

For more advanced debugging scenarios, consider using:

1. UART logging to an external terminal
2. JTAG/SWD hardware debugger connected to Teensy debug port
3. Additional GPIO pins configured as debug outputs
4. Runtime event logging to a reserved section of RAM for post-crash analysis

### New Diagnostic Instrumentation

The updated firmware includes enhanced diagnostic capabilities:

#### USB Device State Monitoring

The USB subsystem includes detailed state transition monitoring:

* Tracks the current USB device state (Default, Address, Configured, Suspended)
* Detects and logs invalid state transitions
* Counts consecutive errors and triggers automatic recovery
* Provides detailed USB error diagnostic information

During USB initialization, the system goes through several state transitions:
1. **Default State**: Initial state after connection or reset
2. **Address State**: Host assigns a unique address to the device
3. **Configured State**: Host successfully configures the device
4. **Suspended State**: Device enters low-power mode (when host suspends)

If you're experiencing panic situations during USB initialization, pay attention to which state transition is failing by monitoring the LED patterns.

To enable enhanced USB diagnostics, build with:

```bash
cargo build --features="usb-diagnostics"
```

#### Error Recovery Mechanisms

The firmware now implements several error recovery mechanisms:

* USB Reset: After detecting too many errors, the device will reset the USB connection
* Safe Mode: If critical errors persist, the device can enter a safe mode with minimal functionality
* Error Logging: Historical error logs are maintained even through resets
* Progressive Recovery: The system attempts least disruptive recovery methods first

#### Common Error Conditions and Troubleshooting

Below are common error conditions and their troubleshooting steps:

| Symptom | Possible Causes | Troubleshooting Steps |
|---------|----------------|------------------------|
| Long-short-long LED pattern followed by disconnect | USB initialization error | 1. Try a different USB cable<br>2. Check for interference<br>3. Verify USB port power<br>4. Check USB descriptor configuration |
| Input buttons "ghost" or trigger unexpectedly | Input debounce or config error | 1. Check button wiring for shorts<br>2. Adjust debounce time in config<br>3. Test inputs one at a time |
| Device repeatedly connects/disconnects | USB state error loop | 1. Disconnect for 10 seconds<br>2. Try a different USB port<br>3. Check build logs for USB descriptor errors<br>4. Verify connected USB devices aren't causing controller conflicts |
| Analog sticks jitter or are unresponsive | Analog calibration error | 1. Check ADC configuration<br>2. Run calibration procedure<br>3. Verify analog input wiring |
| Controller works but gives InitError at random times | Unstable initialization | 1. Check power supply stability<br>2. Look for timing issues in initialization<br>3. Verify hardware compatibility |

For each of these issues, the diagnostic instrumentation will provide more detailed information when enabled with the `usb-diagnostics` feature.

##### USB Panic Troubleshooting Checklist

If your device is exhibiting the USB error pattern (long-short-long):

1. **Check hardware connections**:
   - Verify the USB cable is securely connected and not damaged
   - Try a different USB port, preferably directly on the motherboard (not through a hub)
   - Check if the controller is receiving sufficient power

2. **Check firmware configuration**:
   - Ensure the USB descriptor configuration matches your hardware
   - Verify the VID/PID values are correctly set and not conflicting with other devices
   - Check that the USB endpoint configuration is appropriate for your controller type

3. **Check host system**:
   - Look for USB driver issues in your operating system
   - Check if similar devices are recognized properly
   - Try the controller on a different computer if possible

4. **Examine USB state transitions**:
   - Build with USB diagnostics enabled to capture detailed logs
   - Identify at which state the failure occurs
   - Look for repeated state transition failures that might indicate timing issues

5. **Recovery options**:
   - If the controller enters a recovery mode, allow it to complete the recovery process
   - In persistent failure cases, reflash the firmware with a verified stable version
   - As a last resort, perform a hardware reset by disconnecting power completely

## Reporting Issues

When reporting issues, please include:

1. The specific LED pattern observed
2. At which initialization stage the failure occurred
3. Any modifications made to the default firmware
4. Hardware configuration details
5. Steps to reproduce the issue

This information will greatly assist in diagnosing and fixing problems in the firmware.