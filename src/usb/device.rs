//! USB device implementation for Nintendo Switch Pro controller
//!
//! This module handles the USB device configuration and communication
//! for the Nintendo Switch Pro controller.

use core::sync::atomic::{AtomicBool, Ordering};
use log::{debug, error, info, warn};
use teensy4_bsp as bsp;
use bsp::hal::usbd::Instances;
use usb_device::class_prelude::*;
use usb_device::{device::{UsbDeviceBuilder, UsbVidPid}, UsbError, prelude::UsbDeviceState};
use usb_device::prelude::UsbDevice;
use usbd_hid::hid_class::{HIDClass, HidCountryCode, HidProtocol, HidSubClass};
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::hid_class::HidClassSettings;

use super::descriptor::{SwitchProReport, SwitchProReportDescriptor};

// Nintendo Switch Pro Controller VID/PID
const NINTENDO_VID: u16 = 0x057E;
const SWITCH_PRO_PID: u16 = 0x2009;

// USB Polling Interval in milliseconds (1ms for low latency)
const USB_POLL_INTERVAL_MS: u8 = 1;

// Output report buffer size
const OUTPUT_REPORT_SIZE: usize = 8;

// Static flag to indicate if device is connected
static DEVICE_CONNECTED: AtomicBool = AtomicBool::new(false);

/// Nintendo Switch Pro Controller USB Device
pub struct SwitchProDevice {
    usb_dev: UsbDevice<'static, bsp::hal::usbd::BusAdapter>,
    hid: HIDClass<'static, bsp::hal::usbd::BusAdapter>,
    output_report_buffer: [u8; OUTPUT_REPORT_SIZE],
    last_report: SwitchProReport,
    is_connected: bool,
    last_state: UsbDeviceState,
    state_transition_errors: u8,
}

// Static buffer for USB endpoint management
static mut EP_MEMORY: bsp::hal::usbd::EndpointMemory<1024> = bsp::hal::usbd::EndpointMemory::new();

// Static endpoint state
static mut EP_STATE: bsp::hal::usbd::EndpointState = bsp::hal::usbd::EndpointState::new();

// Global reference to the USB bus allocator
static mut USB_BUS: Option<UsbBusAllocator<bsp::hal::usbd::BusAdapter>> = None;

/// Initialize the USB bus allocator
fn init_usb_bus(usb: Instances<1>) -> &'static UsbBusAllocator<bsp::hal::usbd::BusAdapter> {
    static INIT: AtomicBool = AtomicBool::new(false);
    
    unsafe {
        if !INIT.swap(true, Ordering::AcqRel) {
            // Create the USB bus allocation - the first call to this initializes the USB
            let bus = bsp::hal::usbd::BusAdapter::new(usb, &EP_MEMORY, &mut EP_STATE);
            USB_BUS = Some(UsbBusAllocator::new(bus));
        }
        
        USB_BUS.as_ref().unwrap()
    }
}

impl SwitchProDevice {
    /// Initialize a real USB device for Nintendo Switch Pro Controller
    pub fn new(usb: Instances<1>) -> Self {
        info!("Initializing Switch Pro Controller USB device (real implementation)");
        
        // Get the USB bus allocator
        let bus_allocator = init_usb_bus(usb);
        
        // Create the HID Class with the Switch Pro Controller descriptor
        // Use the simpler constructor without complex settings that's causing issues
        let hid = HIDClass::new(
            &bus_allocator,
            SwitchProReportDescriptor::desc(),
            USB_POLL_INTERVAL_MS
        );
        
        // Build the USB device with Switch Pro Controller VID/PID
        let usb_dev = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(NINTENDO_VID, SWITCH_PRO_PID))
            .manufacturer("Nintendo")
            .product("Pro Controller")
            .serial_number("000000000001")
            .device_class(0) // Use class from interface
            .max_packet_size_0(64) // Use maximum packet size
            .max_power(500) // 500 mA
            .build();
        
        debug!("USB device and HID class initialized");
        
        Self {
            usb_dev,
            hid,
            output_report_buffer: [0; OUTPUT_REPORT_SIZE],
            last_report: SwitchProReport::new(),
            is_connected: false,
            last_state: UsbDeviceState::Default,
            state_transition_errors: 0,
        }
    }
    
    /// Send a report to the Switch
    pub fn send_report(&mut self, report: &SwitchProReport) -> Result<(), UsbError> {
        // Store the report for reference
        self.last_report = *report;
        
        // Get the raw bytes
        let report_bytes = report.to_bytes();
        
        debug!("Sending controller report bytes: {:?}", report_bytes);
        
        // Only attempt to send if the device is configured
        if self.usb_dev.state() == UsbDeviceState::Configured {
            // Use the push_raw_input method to send raw bytes instead
            match self.hid.push_raw_input(&report_bytes) {
                Ok(_) => {
                    debug!("Report sent successfully");
                    Ok(())
                },
                Err(UsbError::WouldBlock) => {
                    // WouldBlock is normal if the host isn't ready for data
                    debug!("USB busy, report not sent");
                    Ok(())
                },
                Err(e) => {
                    warn!("Failed to send USB report: {:?}", e);
                    Err(e)
                }
            }
        } else {
            // Device not configured yet, just return Ok
            debug!("Device not configured, report not sent");
            Ok(())
        }
    }
    
    /// Poll for USB events and handle state transitions
    pub fn poll(&mut self) -> Result<(), UsbError> {
        // Poll the USB device to handle control transfers
        let _ = self.usb_dev.poll(&mut [&mut self.hid]);
        
        // Get the current device state
        let current_state = self.usb_dev.state();
        
        // Track state transitions for error detection and recovery
        if self.last_state != current_state {
            debug!("USB device state changed: {:?} -> {:?}", self.last_state, current_state);
            
            // Track valid/invalid state transitions
            match (self.last_state, current_state) {
                (UsbDeviceState::Configured, UsbDeviceState::Default) => {
                    // Unexpected disconnect detected - likely cable issue or host reset
                    warn!("USB device unexpectedly disconnected");
                    self.state_transition_errors += 1;
                },
                (UsbDeviceState::Default, UsbDeviceState::Configured) => {
                    // Connected to host
                    info!("USB device connected to host");
                    self.state_transition_errors = 0; // Reset error counter
                },
                _ => {
                    // Other state transition
                    debug!("USB state transition: {:?} -> {:?}", self.last_state, current_state);
                }
            }
            
            // Update connection status and last state
            self.is_connected = current_state == UsbDeviceState::Configured;
            self.last_state = current_state;
            
            // Update the global connected flag
            DEVICE_CONNECTED.store(self.is_connected, Ordering::SeqCst);
        }
        
        // Automatic recovery for too many errors
        if self.state_transition_errors > 5 {
            error!("Too many USB state transition errors, attempting recovery");
            self.reset();
            self.state_transition_errors = 0;
            return Err(UsbError::WouldBlock);
        }
        
        // Process any output reports from the host
        self.process_output_reports();
        
        // Return Ok to indicate we've successfully polled
        Ok(())
    }
    
    /// Get the current USB device state
    fn get_device_state(&self) -> UsbDeviceState {
        self.usb_dev.state()
    }
    
    /// Process any output reports from the host
    fn process_output_reports(&mut self) {
        // Only attempt to read if the device is configured
        if self.usb_dev.state() == UsbDeviceState::Configured {
            // Try to read an output report from the host
            match self.hid.pull_raw_output(&mut self.output_report_buffer) {
                Ok(size) => {
                    if size > 0 {
                        debug!("Received output report from host: {:?}", &self.output_report_buffer[..size]);
                        // Process the report (e.g., rumble, LED settings)
                        // Create a copy of the buffer to avoid borrowing issues
                        let buffer_copy = self.output_report_buffer;
                        self.handle_output_report(&buffer_copy[..size]);
                    }
                },
                Err(UsbError::WouldBlock) => {
                    // No data available, this is normal
                },
                Err(e) => {
                    warn!("Error reading output report: {:?}", e);
                }
            }
        }
    }
    
    /// Handle an output report from the host
    fn handle_output_report(&mut self, report: &[u8]) {
        // In a full implementation, this would process commands from the Switch
        // such as rumble data, LED settings, etc.
        if !report.is_empty() {
            debug!("Processing output report: {:?}", report);
            // For now, just log the report
        }
    }
    
    /// Get the connection status
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }
    
    /// Get a static global connection status
    pub fn is_device_connected() -> bool {
        DEVICE_CONNECTED.load(Ordering::SeqCst)
    }
    
    /// Reset the device in case of critical errors
    pub fn reset(&mut self) {
        info!("Resetting USB device");
        // Reset the device state
        self.is_connected = false;
        self.last_state = UsbDeviceState::Default;
        self.state_transition_errors = 0;
        
        // Clear buffers
        self.output_report_buffer = [0; OUTPUT_REPORT_SIZE];
        
        // The actual USB hardware reset would happen here if we had direct access
        // For now, we'll rely on the next poll cycle to re-establish connection
    }
}