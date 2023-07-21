use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_backtrace as _;
use hal::gpio::{Input, Output, Pins, PullUp, PushPull, AnyPin};
use keyberon::key_code::KeyCode;
use keyberon::matrix::Matrix;
use keyberon::{action::k, debounce::Debouncer, layout::Layers};

use KeyCode::*;
static LAYOUT: Layers<6, 3, 1> = [[
    [k(A), k(B), k(C), k(D), k(E), k(F)],
    [k(G), k(H), k(I), k(J), k(K), k(L)],
    [k(M), k(N), k(O), k(P), k(Q), k(R)],
]];


pub struct BoardLeftFinger<C: InputPin, R: OutputPin> {
    pub matrix: Matrix<C, R, 6, 3>,
    pub debouncer: Debouncer<[[bool; 6]; 3]>,
    pub layout: Layers<6, 3, 1>,
}

impl BoardLeftFinger<AnyPin<Input<PullUp>>, AnyPin<Output<PushPull>>> {
    /// The consumed IO is returned
    /// ```ignore
    /// let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    /// let (board_left_finger, io) = left_finger::BoardLeftFinger::new(io);
    /// ```
    pub fn new(pins: Pins) -> Self {
        let matrix = Matrix::new(
            [
                pins.gpio37.into_pull_up_input().degrade(),
                pins.gpio38.into_pull_up_input().degrade(),
                pins.gpio39.into_pull_up_input().degrade(),
                pins.gpio40.into_pull_up_input().degrade(),
                pins.gpio41.into_pull_up_input().degrade(),
                pins.gpio42.into_pull_up_input().degrade(),
            ],
            [
                // pins.gpio21.into_push_pull_output(),
                pins.gpio47.into_push_pull_output().degrade(),
                pins.gpio48.into_push_pull_output().degrade(),
                pins.gpio45.into_push_pull_output().degrade(),
            ],
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
