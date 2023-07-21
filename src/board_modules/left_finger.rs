use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_backtrace as _;
use hal::gpio::{AnyPin, Input, Output, Pins, PullUp, PushPull};
use keyberon::key_code::KeyCode;
use keyberon::matrix::Matrix;
use keyberon::{action::k, debounce::Debouncer, layout::Layers};

use KeyCode::*;
#[rustfmt::skip]
static LAYOUT: Layers<4, 6, 1> = [[
    [k(No),     k(No), k(A), k(Z)],
    [k(No),     k(Q), k(S), k(X)],
    [k(Escape), k(W), k(D), k(C)],
    [k(LCtrl),  k(E), k(F), k(V)],
    [k(LAlt),   k(R), k(G), k(B)],
    [k(No),     k(T), k(Q), k(R)],
]];

/// Note: keyberon::matrix::Matrix assumes input is column,
/// while lynx-cat hardware has row as input.
pub struct BoardLeftFinger<C: InputPin, R: OutputPin> {
    pub matrix: Matrix<C, R, 4, 6>,
    pub debouncer: Debouncer<[[bool; 4]; 6]>,
    pub layout: Layers<4, 6, 1>,
}

impl BoardLeftFinger<AnyPin<Input<PullUp>>, AnyPin<Output<PushPull>>> {
    pub fn new(pins: Pins) -> Self {
        let matrix = Matrix::new(
            [
                pins.gpio21.into_pull_up_input().degrade(),
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
        let debounce = || [[false; 4]; 6];

        Self {
            matrix,
            debouncer: Debouncer::new(debounce(), debounce(), 50),
            layout: LAYOUT,
        }
    }
}
