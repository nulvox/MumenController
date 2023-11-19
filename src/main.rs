#![no_std]
#![no_main]

use arduino_hal;
use panic_halt as _;

// Button state masks
static MASK_A: u16 = 0x0004;
static MASK_B: u16 = 0x0002;
static MASK_X: u16 = 0x0008;
static MASK_Y: u16 = 0x0001;
static MASK_L1: u16 = 0x0010;
static MASK_R1: u16 = 0x0020;
static MASK_L2: u16 = 0x0040;
static MASK_R2: u16 = 0x0080;
static MASK_SELECT: u16 = 0x0100;
static MASK_START: u16 = 0x0200;
static MASK_HOME: u16 = 0x1000;
static MASK_NONE: u16 = 0x0000;

// Dpad Hat switch state masks
static PAD_MASK_UP: u8 = 0x00;
static PAD_MASK_UPRIGHT: u8 = 0x01;
static PAD_MASK_RIGHT: u8 = 0x02;
static PAD_MASK_DOWNRIGHT: u8 = 0x03;
static PAD_MASK_DOWN: u8 = 0x04;
static PAD_MASK_DOWNLEFT: u8 = 0x05;
static PAD_MASK_LEFT: u8 = 0x06;
static PAD_MASK_UPLEFT: u8 = 0x07;
static PAD_MASK_NONE: u8 = 0x08;

// @TODO make this react to the pinout_config contents
fn configure_input_pins(pinout_config) -> [&dyn InputPin<Error = core::convert::Infallible>; 16] {
    let pins = arduino_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut input_pins = [
        pins.gpio0.into_pull_down_input(),
        pins.gpio1.into_pull_down_input(),
        pins.gpio2.into_pull_down_input(),
        pins.gpio3.into_pull_down_input(),
        pins.gpio4.into_pull_down_input(),
        pins.gpio5.into_pull_down_input(),
        pins.gpio6.into_pull_down_input(),
        pins.gpio7.into_pull_down_input(),
        pins.gpio8.into_pull_down_input(),
        pins.gpio9.into_pull_down_input(),
        pins.gpio10.into_pull_down_input(),
        pins.gpio11.into_pull_down_input(),
    ];

    input_pins
}

read_pinout_config(pinout_config_path) -> ButtonPinout {
    let mut file = File::open(pinout_config_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let button_pinout: ButtonPinout = toml::from_str(&contents).unwrap();
    button_pinout
}

fn get_report(pins: &[&dyn InputPin<Error = core::convert::Infallible>; 12]) -> SwitchGamepadReport {
    // Read out 8 buttons first
    let mut buttons = 0;
    for (idx, &pin) in pins[..8].iter().enumerate() {
        if pin.is_low().unwrap() {
            buttons |= 1 << idx;
        }
    }
    // @TODO actually get the values from our input pins

    // We're using digital switches in a D-PAD style configuration
    //    10
    //  8    9
    //    11
    // These are mapped to the limits of an axis
    // let x = if pins[8].is_low().unwrap() {
    //     -127 // left
    // } else if pins[9].is_low().unwrap() {
    //     127 // right
    // } else {
    //     0 // center
    // };

    // let y = if pins[10].is_low().unwrap() {
    //     -127 // up
    // } else if pins[11].is_low().unwrap() {
    //     127 // down
    // } else {
    //     0 // center
    // };

    SwitchGamepadReport { buttons, x, y }
}

#[arduino_hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("Starting");

    //USB
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut mumen = UsbHidClassBuilder::new()
        .add_device(
            usbd_human_interface_device::device::switch_gamepad::SwitchGamepadConfig::default(),
        )
        .build(&usb_bus);

    //https://pid.codes
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .manufacturer("usbd-human-interface-device")
        .product("Mumen Switch Controller")
        .serial_number("0")
        .build();

    let mut led_pin = pins.gpio13.into_push_pull_output();
    led_pin.set_high().ok();
    
    let pinout_config_path = "switchspec/d16s.toml";
    let button_pinout = read_pinout_config(pinout_config_path);
    //@TODO make this function work...
    let input_pins = configure_input_pins(button_pinout);
    input_copunt_down.start(1.millis());

    loop {
        // Poll every 10ms
        if input_count_down.wait().is_ok() {
            match mumen.device().write_report(&get_report(&input_pins)) {
                Err(UsbHidError::WouldBlock) => {}
                Ok(_) => {}
                Err(e) => {
                    core::panic!("Failed to write joystick report: {:?}", e)
                }
            }
        }

        if usb_dev.poll(&mut [&mut mumen]) {}
    }
}
