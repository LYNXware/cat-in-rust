#![warn(unused_imports)]
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::logger::init_logger;
use hal::prelude::*;
use hal::{clock::ClockControl, peripherals::Peripherals, timer::TimerGroup, Rtc, IO};

use keyberon::layout::Event;

mod board_modules;
use board_modules::left_finger;

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Trace);
    log::trace!("entered main, logging initialized");
    // Obtain board resources
    let peripherals = Peripherals::take();
    log::trace!("Peripherals claimed");
    let mut system = peripherals.SYSTEM.split();
    log::trace!("System components claimed");
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    log::trace!("clocks claimed with boot-defaultes, and frozen");

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
    log::trace!("Disabling timers:");
    rtc.rwdt.disable();

    log::trace!("\t rtc.rwdt.disable()");
    wdt0.disable();
    log::trace!("\t wdt0.disable()");
    wdt1.disable();
    log::trace!("\t wdt1.disable()");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    log::trace!("gpio created");
    // Setup the board-left-finger module
    let mut board_left_finger = left_finger::BoardLeftFinger::new(io.pins);

    // Demonstrate PoC using keyberon to read the matrix
    log::info!("Begin reading keyboard");
    for _ in 0.. {
        let events = board_left_finger
            .debouncer
            .events(
                board_left_finger.matrix.down_keys().unwrap(),
                Some(keyberon::debounce::transpose),
            )
            .collect::<heapless::Vec<_, 8>>();
        for event in events.iter() {
            match event {
                Event::Press(x, y) => log::info!(
                    "P-{:?}",
                    board_left_finger.layout[0][*x as usize][*y as usize]
                ),
                Event::Release(x, y) => log::info!(
                    "R-{:?}",
                    board_left_finger.layout[0][*x as usize][*y as usize]
                ),
            }
        }
    }
    unreachable!()
}
