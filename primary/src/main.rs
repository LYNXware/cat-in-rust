#![warn(unused_imports)]
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    efuse::Efuse,
    otg_fs::{UsbBus, USB},
    peripherals::Peripherals,
    uart::{config::Config as UartConfig, TxRxPins as UartTxRx, Uart},
    Rng, IO,
};
use esp_hal::{prelude::*, Delay};
use esp_println::logger::init_logger;
use esp_wifi::{
    esp_now::{PeerInfo, BROADCAST_ADDRESS},
    EspWifiInitFor,
};
use generic_array::GenericArray;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};

use usbd_human_interface_device::device::{
    keyboard::{BootKeyboard, BootKeyboardConfig},
    mouse::WheelMouseConfig,
};
use usbd_human_interface_device::prelude::*;
// imports for wheel mouse. implied TODO, of course
// use keyberon::key_code::KeyCode;
// use usbd_human_interface_device::device::mouse::{WheelMouseReport, WheelMouse};
// use components::mouse::{MouseWheelDriver, Scroller, UninitWheelPins};

use components::{
    matrix::{KeyDriver, UninitKeyPins},
    ReadState,
};

mod hardware;

static mut USB_MEM: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);
    log::trace!("entered main, logging initialized");
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let timer = esp_hal::timer::TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    )
    .timer0;
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

    // let usb = USB::new(
    //     peripherals.USB0,
    //     io.pins.gpio18,
    //     io.pins.gpio19,
    //     io.pins.gpio20,
    //     &mut system.peripheral_clock_control,
    // );

    // let usb_bus = UsbBus::new(usb, unsafe { &mut USB_MEM });
    // let mut classes = UsbHidClassBuilder::new()
    //     .add_device(WheelMouseConfig::default())
    //     .add_device(BootKeyboardConfig::default())
    //     .build(&usb_bus);

    // let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0, 0))
    //     .manufacturer("shady-bastard")
    //     .product("totally not a malicious thing")
    //     .device_class(3)
    //     .build();
    // usb_dev.poll(&mut [&mut classes]);

    let wifi_init = esp_wifi::initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();
    let (wifi, ..) = peripherals.RADIO.split();
    let left_finger = UninitKeyPins {
        ins: GenericArray::from_array([
            io.pins.gpio38.into_pull_up_input().degrade(),
            io.pins.gpio37.into_pull_up_input().degrade(),
            io.pins.gpio36.into_pull_up_input().degrade(),
            io.pins.gpio35.into_pull_up_input().degrade(),
        ]),
        outs: GenericArray::from_array([
            io.pins.gpio44.into_push_pull_output().degrade(),
            io.pins.gpio1.into_push_pull_output().degrade(),
            io.pins.gpio2.into_push_pull_output().degrade(),
            io.pins.gpio42.into_push_pull_output().degrade(),
            io.pins.gpio41.into_push_pull_output().degrade(),
            io.pins.gpio40.into_push_pull_output().degrade(),
            io.pins.gpio39.into_push_pull_output().degrade(),
        ]),
    };

    let mut left_finger = KeyDriver::new(left_finger, Delay::new(&clocks));
    let left_thumb = UninitKeyPins {
        ins: GenericArray::from_array([
            io.pins.gpio17.into_pull_up_input().degrade(),
            io.pins.gpio16.into_pull_up_input().degrade(),
            io.pins.gpio15.into_pull_up_input().degrade(),
            io.pins.gpio7.into_pull_up_input().degrade(),
        ]),
        outs: GenericArray::from_array([
            io.pins.gpio4.into_push_pull_output().degrade(),
            io.pins.gpio5.into_push_pull_output().degrade(),
            io.pins.gpio6.into_push_pull_output().degrade(),
        ]),
    };
    let lf_matrix_len = left_finger.bit_len();
    // let mut left_thumb = KeyDriver::new(left_thumb, Delay::new(&clocks));

    // pin place-holders for now. refer to wiring diagram for correction
    // let pin_a = io.pins.gpio43.into_pull_up_input();
    // let pin_b = io.pins.gpio45.into_pull_up_input();
    // let gnd = io.pins.gpio0.into_push_pull_output();
    // let wheel_pins = UninitWheelPins {
    //     in1: pin_a,
    //     in2: pin_b,
    //     gnd: Some(gnd),
    // };
    // let mut wheel = MouseWheelDriver::new(wheel_pins);

    // let mut delay = Delay::new(&clocks);
    // let mut esp_now = esp_wifi::esp_now::EspNow::new(&wifi_init, wifi).unwrap();
    // let secondary = PeerInfo{ peer_address: BROADCAST_ADDRESS, lmk: None, channel: None, encrypt: false };
    // esp_now.add_peer(secondary).unwrap();
    let mut lf_state = GenericArray::default();
    // let mut lt_state = GenericArray::default();
    loop {
        // let scroll = wheel.read_scroll();
        left_finger.read_state(&mut lf_state);
        log::info!("{:?}", &lf_state[0..((lf_matrix_len + 7) / 8)]);
        // left_thumb.read_state(&mut lt_state);
        // TODO: check for state from secondary...

        // TODO: transform the state into a kb-report
        // let keyboard = classes.device::<BootKeyboard<'_, _>, _>();
        // match keyboard.write_report(kb_report) {
        //     Err(UsbHidError::WouldBlock | UsbHidError::Duplicate) | Ok(_) => {}
        //     Err(e) => {
        //         core::panic!("Failed to write keyboard report: {:?}", e)
        //     }
        // };
        // if let Some(lf_rep) = esp_now.receive() {
        //     log::info!("{:?}", &lf_rep.data[0..(lf_rep.len as usize)]);
        // }
        // usb_dev.poll(&mut [&mut classes]);
    }
}
