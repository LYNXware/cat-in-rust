#![warn(unused_imports)]
#![no_std]
#![no_main]

use bitvec::{order::Lsb0, slice::BitSlice};
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
use esp_wifi::{esp_now::PeerInfo, EspWifiInitFor};
use generic_array::typenum::Unsigned;
use keyberon::layout::Layout;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};

use usbd_human_interface_device::device::{
    keyboard::{BootKeyboard, BootKeyboardConfig},
    mouse::WheelMouseConfig,
};
use usbd_human_interface_device::prelude::*;
// imports for wheel mouse. implied TODO, of course
// use keyberon::key_code::KeyCode;
// use usbd_human_interface_device::device::mouse::{WheelMouseReport, WheelMouse};


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

    let mut delay = Delay::new(&clocks);
    let esp_now = esp_wifi::esp_now::EspNow::new(&wifi_init, wifi).unwrap();
    let _ = esp_now
        .add_peer(PeerInfo {
            peer_address: LEFT,
            lmk: None,
            channel: None,
            encrypt: false,
        })
        .unwrap();
    let _ = esp_now
        .add_peer(PeerInfo {
            peer_address: RIGHT,
            lmk: None,
            channel: None,
            encrypt: false,
        })
        .unwrap();

    loop {
        if let Some(rf) = esp_now.receive() {
            let msg: &[u8] = &rf.data[0..(rf.len as usize)];
            if rf.info.src_address == LEFT {
                log::info!("{:?}", to_bool_thing::<4, 6>(msg));
            }
        }

        // TODO: fuse all the data and write the usb reports
        // TODO: recieve reconfiguration instructions

        // TODO: transform the state into a kb-report
        // let keyboard = classes.device::<BootKeyboard<'_, _>, _>();
        // match keyboard.write_report(kb_report) {
        //     Err(UsbHidError::WouldBlock | UsbHidError::Duplicate) | Ok(_) => {}
        //     Err(e) => {
        //         core::panic!("Failed to write keyboard report: {:?}", e)
        //     }
        // };
        // usb_dev.poll(&mut [&mut classes]);
        delay.delay_ms(1u32);
    }
}

fn to_bool_thing<const PRE_W: usize, const PRE_H: usize>(bytes: &[u8]) -> [[bool; PRE_H]; PRE_W] {
    let mut res = [[false; PRE_H]; PRE_W];
    for idx in 0..(PRE_W * PRE_H) {
        let row = idx / PRE_W;
        let col = idx % PRE_W;
        let byte = idx / 8;
        let bit = idx % 8;
        res[col][PRE_H - row - 1] = bytes[byte] & (1 << bit) > 0;
    }
    res
}
