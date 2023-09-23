#![warn(unused_imports)]
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    efuse::Efuse,
    peripherals::Peripherals,
    uart::{config::Config as UartConfig, TxRxPins as UartTxRx, Uart},
    IO,
};
use esp_hal::{prelude::*, Delay, Rng};
use esp_println::logger::init_logger;
use esp_wifi::{esp_now::BROADCAST_ADDRESS, EspWifiInitFor};

// imports for wheel mouse. implied TODO, of course
use components::{mouse::{MouseWheelDriver, Scroller, UninitWheelPins}, ReadState};
use generic_array::GenericArray;
use keyberon::key_code::KeyCode;

use components::matrix::{KeyDriver, UninitKeyPins};

mod hardware;

/* TODO: setup some sort of configuration-based hard-coding to set the primary.
 *  - needs to be statically defined. Zero-runtime
 *  - defined in such a way that works well with a git-repo
 *  - one idea: write a tool that gets the MAC address of the primary/others, and generates a
 *  config file (tson, toml, json, etc.) which a build.rs can use to compile-time define things
*/
static PRIMARY_ADDR: [u8; 6] = BROADCAST_ADDRESS;

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);
    log::trace!("entered main, logging initialized");
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    log::info!("MAC address {:02x?}", Efuse::get_mac_address());

    let timer = esp_hal::timer::TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    )
    .timer0;
    let wifi_init = esp_wifi::initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();
    let (wifi, ..) = peripherals.RADIO.split();
    let mut esp_now = esp_wifi::esp_now::EspNow::new(&wifi_init, wifi).unwrap();

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

    let right_finger = UninitKeyPins {
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

    let mut right_finger = KeyDriver::new(
        right_finger,
        Delay::new(&clocks),
    );
    let right_thumb = UninitKeyPins {
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
    let mut right_thumb = KeyDriver::new(
        right_thumb,
        Delay::new(&clocks),
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
    let mut rf_state = GenericArray::default();
    let mut rt_state = GenericArray::default();
    loop {
        let scroll = wheel.read_scroll();
        right_finger.read_state(&mut rf_state);
        right_thumb.read_state(&mut rt_state);

        if let Some(scroll) = scroll {
            let scroll = match scroll {
                KeyCode::MediaScrollDown => 1,
                KeyCode::MediaScrollUp => -1,
                _ => panic!("this shouldn't happen"),
            };
        }
        delay.delay_us(300u32);
        todo!("esp-now send the events");
    }
}
