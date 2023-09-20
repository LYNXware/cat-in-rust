#![warn(unused_imports)]
#![no_std]
#![no_main]

use embedded_hal::digital::v2::OutputPin;
use esp_backtrace as _;
use esp_println::logger::init_logger;
use hal::otg_fs::{UsbBus, USB};
use hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    timer::TimerGroup,
    uart::{config::Config as UartConfig, TxRxPins as UartTxRx, Uart},
    Rtc, IO,
};
use hal::{prelude::*, Delay};
use keyberon::key_code::KeyCode;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};

use usbd_human_interface_device::device::mouse::{WheelMouse, WheelMouseReport};
use usbd_human_interface_device::device::{
    keyboard::{BootKeyboard, BootKeyboardConfig},
    mouse::WheelMouseConfig,
};
use usbd_human_interface_device::prelude::*;

use crate::hardware::matrix::{KeyDriver, UninitKeyPins};
use crate::hardware::wheel::{MouseWheelDriver, Scroller};

mod board_modules;
mod hardware;

static mut USB_MEM: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);
    log::trace!("entered main, logging initialized");
    let peripherals = Peripherals::take();
    log::trace!("Peripherals claimed");
    let mut system = peripherals.SYSTEM.split();
    log::trace!("System components claimed");
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    log::trace!("clocks claimed with boot-defaultes, and frozen");
    log::info!("Board resources claimed: peripherals, system, clocks");

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    log::trace!("Rtc claimed");
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    log::trace!("TimerGroup0 created");
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    log::trace!("TimerGroup1 created");
    let mut wdt1 = timer_group1.wdt;
    rtc.rwdt.disable();

    log::trace!("\t rtc.rwdt.disable()");
    wdt0.disable();
    log::trace!("\t wdt0.disable()");
    wdt1.disable();
    log::trace!("\t wdt1.disable()");
    log::info!("clocks configured: rtc-wdt, wdt0/1 disabled");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
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

    log::info!("uart-setup: gnd: gnd, tx: 44, rx: 43, pwr: 1");

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

    let left_finger = UninitKeyPins {
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

    let mut left_finger = KeyDriver::new(
        left_finger,
        5,
        Delay::new(&clocks),
        &board_modules::left_finger::LAYERS,
    );
    let left_thumb = UninitKeyPins {
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
    let mut left_thumb = KeyDriver::new(
        left_thumb,
        5,
        Delay::new(&clocks),
        &board_modules::left_thumb::LAYERS,
    );

    // let pin_a = io.pins.gpio35.into_pull_up_input();
    // let pin_b = io.pins.gpio36.into_pull_up_input();
    // let gnd = io.pins.gpio0.into_push_pull_output();
    // let wheel_pins = hardware::wheel::UninitWheelPins {
    //     in1: pin_a,
    //     in2: pin_b,
    //     gnd: Some(gnd),
    // };
    // let mut wheel = MouseWheelDriver::new(wheel_pins);

    let mut delay = Delay::new(&clocks);
    loop {
        // let scroll = wheel.read_scroll();
        let lf_report = left_finger.events();
        let kb_report = lf_report.chain(left_thumb.events());
        let keyboard = classes.device::<BootKeyboard<'_, _>, _>();

        match keyboard.write_report(kb_report) {
            Err(UsbHidError::WouldBlock | UsbHidError::Duplicate) | Ok(_) => {}
            Err(e) => {
                core::panic!("Failed to write keyboard report: {:?}", e)
            }
        };
        // if let Some(scroll) = scroll {
        //     let scroll = match scroll {
        //         KeyCode::MediaScrollDown => 1,
        //         KeyCode::MediaScrollUp => -1,
        //         _ => panic!("this shouldn't happen"),
        //     };
        //     let mouse_report = WheelMouseReport {
        //         buttons: 0,
        //         x: 0,
        //         y: 0,
        //         vertical_wheel: scroll,
        //         horizontal_wheel: 0,
        //     };
        //     let mouse = classes.device::<WheelMouse<'_, _>, _>();
        //     match mouse.write_report(&mouse_report) {
        //         Err(UsbHidError::WouldBlock) | Ok(_) => {}
        //         Err(e) => {
        //             core::panic!("Failed to write mouse report: {:?}", e)
        //         }
        //     };
        // }
        delay.delay_us(300u32);
        usb_dev.poll(&mut [&mut classes]);
    }
}
