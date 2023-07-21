use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_backtrace as _;
use generic_array::typenum::{U3, U6};
use hal::gpio::{self, Input, Output, Pins, PullUp, PushPull};
use keyberon::impl_heterogenous_array;
use keyberon::key_code::KeyCode;
use keyberon::matrix::Matrix;
use keyberon::{action::k, debounce::Debouncer, layout::Layers};

/// Physical pins corresponding to left-finger module, oriented left-to-right
pub struct LeftFingerCol(
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
pub struct LeftFingerRow(
    // gpio::Gpio21<Output<PushPull>>,
    gpio::Gpio47<Output<PushPull>>,
    gpio::Gpio48<Output<PushPull>>,
    gpio::Gpio45<Output<PushPull>>,
);
use KeyCode::*;
static LAYOUT: Layers = &[&[
    &[k(A), k(B), k(C), k(D), k(E), k(F)],
    &[k(G), k(H), k(I), k(J), k(K), k(L)],
    &[k(M), k(N), k(O), k(P), k(Q), k(R)],
]];

impl_heterogenous_array! {
    LeftFingerRow,
    dyn OutputPin<Error = Infallible>,
    U3,
    [0, 1, 2]
}

pub struct BoardLeftFinger {
    pub matrix: Matrix<LeftFingerCol, LeftFingerRow>,
    pub debouncer: Debouncer<[[bool; 6]; 3]>,
    pub layout: Layers,
}

impl BoardLeftFinger {
    /// The consumed IO is returned
    /// ```ignore
    /// let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    /// let (board_left_finger, io) = left_finger::BoardLeftFinger::new(io);
    /// ```
    pub fn new(pins: Pins) -> Self {
        let matrix = Matrix::new(
            LeftFingerCol(
                pins.gpio37.into_pull_up_input(),
                pins.gpio38.into_pull_up_input(),
                pins.gpio39.into_pull_up_input(),
                pins.gpio40.into_pull_up_input(),
                pins.gpio41.into_pull_up_input(),
                pins.gpio42.into_pull_up_input(),
            ),
            LeftFingerRow(
                // pins.gpio21.into_push_pull_output(),
                pins.gpio47.into_push_pull_output(),
                pins.gpio48.into_push_pull_output(),
                pins.gpio45.into_push_pull_output(),
            ),
        )
        .unwrap();
        let debounce = || [[false; 6]; 3];

        Self {
            matrix,
            debouncer: Debouncer::new(debounce(), debounce(), 5),
            layout: LAYOUT,
        }
    }
}
