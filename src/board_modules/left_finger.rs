use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_backtrace as _;
use generic_array::typenum::{U4, U6};
use hal::gpio::{Input, Output, PullUp, PushPull};
use hal::{gpio, IO};
use keyberon::debounce::Debouncer;
use keyberon::impl_heterogenous_array;
use keyberon::matrix::Matrix;

/// Physical pins corresponding to left-finger module, oriented left-to-right
struct LeftFingerCol(
    gpio::Gpio37<Input<PullUp>>,
    gpio::Gpio38<Input<PullUp>>,
    gpio::Gpio39<Input<PullUp>>,
    gpio::Gpio40<Input<PullUp>>,
    gpio::Gpio41<Input<PullUp>>,
    gpio::Gpio42<Input<PullUp>>,
);
impl_heterogenous_array! {
    LeftFingerCol,
    dyn InputPin<Error = Infallible>,
    U6,
    [0, 1, 2, 3, 4, 5]
}
/// Physical pins corresponding to left-finger module, oriented top-to-bottom
struct LeftFingerRow(
    gpio::Gpio21<Output<PushPull>>,
    gpio::Gpio47<Output<PushPull>>,
    gpio::Gpio48<Output<PushPull>>,
    gpio::Gpio45<Output<PushPull>>,
);

impl_heterogenous_array! {
    LeftFingerRow,
    dyn OutputPin<Error = Infallible>,
    U4,
    [0, 1, 2, 3]
}

pub struct BoardLeftFinger {
    matrix: Matrix<LeftFingerCol, LeftFingerRow>,
    debouncer: Debouncer<[[bool; 6]; 4]>,
}

impl BoardLeftFinger {
    /// The consumed IO is returned
    /// ```ignore
    /// let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    /// let (board_left_finger, io) = left_finger::BoardLeftFinger::new(io);
    /// ```
    pub fn new(io: IO) -> (Self, IO) {
        let matrix = Matrix::new(
            LeftFingerCol(
                io.pins.gpio37.into_pull_up_input(),
                io.pins.gpio38.into_pull_up_input(),
                io.pins.gpio39.into_pull_up_input(),
                io.pins.gpio40.into_pull_up_input(),
                io.pins.gpio41.into_pull_up_input(),
                io.pins.gpio42.into_pull_up_input(),
            ),
            LeftFingerRow(
                io.pins.gpio21.into_push_pull_output(),
                io.pins.gpio47.into_push_pull_output(),
                io.pins.gpio48.into_push_pull_output(),
                io.pins.gpio45.into_push_pull_output(),
            ),
        )
        .unwrap();
        let debounce = || [[false; 6]; 4];

        (
            Self {
                matrix,
                debouncer: Debouncer::new(debounce(), debounce(), 5),
            },
            io,
        )
    }
}
