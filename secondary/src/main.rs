#![warn(unused_imports)]
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::otg_fs::{UsbBus, USB};
use esp_hal::{
    clock::ClockControl,
    efuse::Efuse,
    peripherals::Peripherals,
    uart::{config::Config as UartConfig, TxRxPins as UartTxRx, Uart},
    IO,
};
use esp_hal::{prelude::*, Delay};
use esp_println::logger::init_logger;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};

use usbd_human_interface_device::device::{
    keyboard::{BootKeyboard, BootKeyboardConfig},
    mouse::WheelMouseConfig,
};
use usbd_human_interface_device::prelude::*;
// imports for wheel mouse. implied TODO, of course
use keyberon::key_code::KeyCode;
use usbd_human_interface_device::device::mouse::{WheelMouseReport, WheelMouse};
use components::mouse::{MouseWheelDriver, Scroller, UninitWheelPins};

use components::matrix::{KeyDriver, UninitKeyPins};

mod hardware;

static mut USB_MEM: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);
    log::trace!("entered main, logging initialized");
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    log::info!("MAC address {:02x?}", Efuse::get_mac_address());

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // uart setup
    let uart_vdd_pin = io.pins.gpio11;
    let mut uart_vdd_pin = uart_vdd_pin.into_push_pull_output();
    uart_vdd_pin.set_high().unwrap();

    let t_r_pins = UartTxRx::new_tx_rx(io.pins.gpio12, io.pins.gpio13);
    let mut gnd = io.pins.gpio14.into_push_pull_output();
    gnd.set_low().unwrap();

    // will log in the background. Don't need to use directly
    let mut _uart = Uart::new_with_config(
        peripherals.UART0,
        Some(UartConfig::default()),
        Some(t_r_pins),
        &clocks,
        &mut system.peripheral_clock_control,
    );

    let usb = USB::new(
        peripherals.USB0,
        io.pins.gpio18,
        io.pins.gpio19,
        io.pins.gpio20,
        &mut system.peripheral_clock_control,
    );

    let usb_bus = UsbBus::new(usb, unsafe { &mut USB_MEM });
    let mut classes = UsbHidClassBuilder::new()
        .add_device(WheelMouseConfig::default())
        .add_device(BootKeyboardConfig::default())
        .build(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0, 0))
        .manufacturer("shady-bastard")
        .product("totally not a malicious thing")
        .device_class(3)
        .build();

    let right_finger = UninitKeyPins {
        ins: [
            io.pins.gpio38.into_pull_up_input().degrade(),
            io.pins.gpio37.into_pull_up_input().degrade(),
            io.pins.gpio36.into_pull_up_input().degrade(),
            io.pins.gpio35.into_pull_up_input().degrade(),
        ],
        outs: [
            io.pins.gpio44.into_push_pull_output().degrade(),
            io.pins.gpio1.into_push_pull_output().degrade(),
            io.pins.gpio2.into_push_pull_output().degrade(),
            io.pins.gpio42.into_push_pull_output().degrade(),
            io.pins.gpio41.into_push_pull_output().degrade(),
            io.pins.gpio40.into_push_pull_output().degrade(),
            io.pins.gpio39.into_push_pull_output().degrade(),
        ],
    };

    let mut right_finger = KeyDriver::new(
        right_finger,
        5,
        Delay::new(&clocks),
        &configs::right_finger::LAYERS,
    );
    let right_thumb = UninitKeyPins {
        ins: [
            io.pins.gpio17.into_pull_up_input().degrade(),
            io.pins.gpio16.into_pull_up_input().degrade(),
            io.pins.gpio15.into_pull_up_input().degrade(),
            io.pins.gpio7.into_pull_up_input().degrade(),
        ],
        outs: [
            io.pins.gpio4.into_push_pull_output().degrade(),
            io.pins.gpio5.into_push_pull_output().degrade(),
            io.pins.gpio6.into_push_pull_output().degrade(),
        ],
    };
    let mut right_thumb = KeyDriver::new(
        right_thumb,
        5,
        Delay::new(&clocks),
        &configs::right_thumb::LAYERS,
    );

    // pin place-holders for now. refer to wiring diagram for correction
    let pin_a = io.pins.gpio45.into_pull_up_input();
    let pin_b = io.pins.gpio48.into_pull_up_input();
    let gnd = io.pins.gpio0.into_push_pull_output();
    let wheel_pins = UninitWheelPins {
        in1: pin_a,
        in2: pin_b,
        gnd: Some(gnd),
    };
    let mut wheel = MouseWheelDriver::new(wheel_pins);

    let mut delay = Delay::new(&clocks);
    loop {
        let scroll = wheel.read_scroll();
        let lf_report = right_finger.events();
        let kb_report = lf_report.chain(right_thumb.events());
        let keyboard = classes.device::<BootKeyboard<'_, _>, _>();

        match keyboard.write_report(kb_report) {
            Err(UsbHidError::WouldBlock | UsbHidError::Duplicate) | Ok(_) => {}
            Err(e) => {
                core::panic!("Failed to write keyboard report: {:?}", e)
            }
        };
        if let Some(scroll) = scroll {
            let scroll = match scroll {
                KeyCode::MediaScrollDown => 1,
                KeyCode::MediaScrollUp => -1,
                _ => panic!("this shouldn't happen"),
            };
            let mouse_report = WheelMouseReport {
                buttons: 0,
                x: 0,
                y: 0,
                vertical_wheel: scroll,
                horizontal_wheel: 0,
            };
            let mouse = classes.device::<WheelMouse<'_, _>, _>();
            match mouse.write_report(&mouse_report) {
                Err(UsbHidError::WouldBlock) | Ok(_) => {}
                Err(e) => {
                    core::panic!("Failed to write mouse report: {:?}", e)
                }
            };
        }
        delay.delay_us(300u32);
        usb_dev.poll(&mut [&mut classes]);
    }
}
