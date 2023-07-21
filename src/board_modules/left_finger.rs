use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_backtrace as _;
use hal::gpio::{AnyPin, Input, Output, Pins, PullUp, PushPull};
use keyberon::key_code::KeyCode;
use keyberon::matrix::Matrix;
use keyberon::{action::k, debounce::Debouncer, layout::Layers};

use KeyCode::*;
static LAYOUT: Layers<3, 6, 1> = [[
    [k(Q), k(A), k(Z)],
    [k(W), k(S), k(X)],
    [k(E), k(D), k(C)],
    [k(R), k(F), k(V)],
    [k(T), k(G), k(B)],
    [k(Y), k(Q), k(R)],
]];

/// Note: keyberon::matrix::Matrix assumes input is column, 
/// while lynx-cat hardware has row as input. 
pub struct BoardLeftFinger<C: InputPin, R: OutputPin> {
    pub matrix: Matrix<C, R, 3, 6>,
    pub debouncer: Debouncer<[[bool; 3]; 6]>,
    pub layout: Layers<3, 6, 1>,
}

impl BoardLeftFinger<AnyPin<Input<PullUp>>, AnyPin<Output<PushPull>>> {
    pub fn new(pins: Pins) -> Self {
        let matrix = Matrix::new(
            [
                pins.gpio47.into_pull_up_input().degrade(),
                pins.gpio48.into_pull_up_input().degrade(),
                pins.gpio45.into_pull_up_input().degrade(),
            ],
            [
                // pins.gpio21.into_push_pull_output(),
                pins.gpio37.into_push_pull_output().degrade(),
                pins.gpio38.into_push_pull_output().degrade(),
                pins.gpio39.into_push_pull_output().degrade(),
                pins.gpio40.into_push_pull_output().degrade(),
                pins.gpio41.into_push_pull_output().degrade(),
                pins.gpio42.into_push_pull_output().degrade(),
            ],
        )
        .unwrap();
        let debounce = || [[false; 3]; 6];

        Self {
            matrix,
            debouncer: Debouncer::new(debounce(), debounce(), 50),
            layout: LAYOUT,
        }
    }
}
