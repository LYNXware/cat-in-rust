#![warn(unused_imports)]
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::prelude::*;
use hal::{clock::ClockControl, peripherals::Peripherals, timer::TimerGroup, Rtc, IO};

mod board_modules;
use board_modules::left_finger;
use keyberon::layout::Event;

#[entry]
fn main() -> ! {
    // Obtain board resources
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    // Setup the board-left-finger module
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut board_left_finger = left_finger::BoardLeftFinger::new(io.pins);

    // Demonstrate PoC using keyberon to read the matrix
    println!("Begin reading keyboard");
    for _ in 0.. {
        let events = board_left_finger
            .debouncer
            .events(board_left_finger.matrix.down_keys().unwrap(), Some(keyberon::debounce::transpose))
            .collect::<heapless::Vec<_, 8>>();
        for event in events.iter() {
            match event {
                Event::Press(x, y) => println!(
                    "P-{:?}",
                    board_left_finger.layout[0][*x as usize][*y as usize]
                ),
                Event::Release(x, y) => println!(
                    "R-{:?}",
                    board_left_finger.layout[0][*x as usize][*y as usize]
                ),
            }
        }
    }
    unreachable!()
}
