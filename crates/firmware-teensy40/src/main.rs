//! Teensy 4.0 Nintendo Switch USB HID fight stick firmware.
//!
//! Uses RTIC v2 with:
//! - Single-owner USB polling inside the `USB_OTG1` hardware interrupt (`usb1`)
//! - Proper `imxrt-usbd` bus configuration (`device.bus().configure()`) upon enumeration
//! - High-Speed `bInterval = 4` ($2^{4-1} \times 125\,\mu\text{s} = 1\,\text{ms}$ / 1000 Hz)
//! - 1 ms `Systick` input scan loop reading real Teensy 4.0 GPIO/ADC pins via `pins::BoardInputs`

#![no_std]
#![no_main]

use teensy4_panic as _;

mod pins;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use crate::pins::BoardInputs;
    use mumen_core::{
        ControllerEngine, SwitchReport, HORI_POKKEN_PID, HORI_VID, REPORT_DESCRIPTOR,
    };
    use rtic_monotonics::systick::{Systick, *};
    use teensy4_bsp::board;
    use teensy4_bsp::hal::usbd::{BusAdapter, EndpointMemory, EndpointState};
    use usb_device::{
        bus::UsbBusAllocator,
        device::{UsbDevice, UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
        UsbError,
    };
    use usbd_hid::hid_class::HIDClass;

    /// In USB 2.0 High-Speed (480 Mbps, default for `imxrt-usbd`), the polling interval
    /// is $2^{\text{bInterval}-1} \times 125\,\mu\text{s}$. `4` yields $8 \times 125\,\mu\text{s} = 1\,\text{ms}$ (1000 Hz).
    const HS_POLL_INTERVAL_1MS: u8 = 4;

    /// Endpoint buffer allocation for `imxrt-usbd`.
    static EP_MEMORY: EndpointMemory<1024> = EndpointMemory::new();
    static EP_STATE: EndpointState = EndpointState::max_endpoints();

    type Bus = BusAdapter;

    #[shared]
    struct Shared {
        /// Latest 8-byte HID input report produced by the 1 ms controller scan task.
        report: SwitchReport,
    }

    #[local]
    struct Local {
        inputs: BoardInputs,
        engine: ControllerEngine,
        hid: HIDClass<'static, Bus>,
        device: UsbDevice<'static, Bus>,
    }

    #[init(local = [bus: Option<UsbBusAllocator<Bus>> = None])]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            usb,
            pins,
            mut gpio1,
            mut gpio2,
            mut gpio4,
            adc1,
            ..
        } = board::t40(cx.device);

        // Configure physical Teensy 4.0 GPIO/ADC pads for the selected layout feature
        let inputs = BoardInputs::new(pins, &mut gpio1, &mut gpio2, &mut gpio4, adc1);

        // Start 1 kHz Systick monotonic for the input polling task
        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        // Initialize the High-Speed USB bus and enable USB interrupts
        let bus = BusAdapter::new(usb, &EP_MEMORY, &EP_STATE);
        bus.set_interrupts(true);
        let bus = cx.local.bus.insert(UsbBusAllocator::new(bus));

        // Allocate the HID class with the 86-byte HORI Pokken / Switch report descriptor
        let hid = HIDClass::new(bus, &REPORT_DESCRIPTOR, HS_POLL_INTERVAL_1MS);

        // Build the USB device with licensed HORI Pokken VID/PID (0x0F0D:0x0092)
        let device = UsbDeviceBuilder::new(bus, UsbVidPid(HORI_VID, HORI_POKKEN_PID))
            .manufacturer("HORI CO.,LTD.")
            .product("POKKEN CONTROLLER")
            .serial_number("MUMEN-T40")
            .max_packet_size_0(64)
            .max_power(500)
            .build();

        let engine = ControllerEngine::default();
        let report = SwitchReport::neutral();

        scan_inputs::spawn().ok();

        (
            Shared { report },
            Local {
                inputs,
                engine,
                hid,
                device,
            },
        )
    }

    /// Periodic 1 ms input scan task.
    ///
    /// Samples and debounces all configured hardware switches (and ADC axes if enabled),
    /// runs SOCD/mode/lock processing via `mumen-core`, and updates the shared USB report.
    #[task(shared = [report], local = [inputs, engine], priority = 1)]
    async fn scan_inputs(mut cx: scan_inputs::Context) {
        loop {
            let logical = cx.local.inputs.sample();
            let next_report = cx.local.engine.update(&logical);

            cx.shared.report.lock(|report| {
                *report = next_report;
            });

            Systick::delay(1.millis()).await;
        }
    }

    /// Hardware USB OTG1 interrupt handler.
    ///
    /// Owns `device` and `hid` exclusively so there is zero lock contention on the USB bus.
    #[task(
        binds = USB_OTG1,
        shared = [report],
        local = [device, hid, configured: bool = false],
        priority = 2
    )]
    fn usb1(mut cx: usb1::Context) {
        let usb1::LocalResources {
            device,
            hid,
            configured,
            ..
        } = cx.local;

        // Poll control & class endpoints
        let _ = device.poll(&mut [hid]);

        if device.state() == UsbDeviceState::Configured {
            // imxrt-usbd requirement: configure non-zero endpoints once after host sets configuration
            if !*configured {
                device.bus().configure();
                *configured = true;
            }

            // Drain any 8-byte output report sent by the Switch so the OUT endpoint never stalls
            let mut out_buf = [0u8; 8];
            let _ = hid.pull_raw_output(&mut out_buf);

            // Push the latest 8-byte input report if the IN endpoint is ready
            let bytes = cx.shared.report.lock(|report| report.to_bytes());
            match hid.push_raw_input(&bytes) {
                Ok(_) | Err(UsbError::WouldBlock) => {}
                Err(_) => {}
            }
        } else {
            *configured = false;
        }
    }
}
