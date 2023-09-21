use core::convert::Infallible;

use embedded_hal::digital::v2::{InputPin, OutputPin};
use keyberon::key_code::KeyCode;

pub struct UninitWheelPins<I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    pub in1: I1,
    pub in2: I2,
    pub gnd: Option<O>,
}

impl<I1, I2, O> UninitWheelPins<I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    fn init(self) -> InitedWheelPins<I1, I2, O> {
        let Self {
            in1: pin_a,
            in2: pin_b,
            gnd,
        } = self;

        let _gnd = gnd.map(|mut g| {
            g.set_low().unwrap();
            g
        });
        InitedWheelPins {
            in1: pin_a,
            in2: pin_b,
            _gnd,
        }
    }
}

pub struct InitedWheelPins<I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    in1: I1,
    in2: I2,
    _gnd: Option<O>,
}

pub struct MouseWheelDriver<I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    pins: InitedWheelPins<I1, I2, O>,
    value: u8,
    state: bool,
    prev_state: bool,
    scroll_val: i8,
}

impl<I1, I2, O> MouseWheelDriver<I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    pub fn new(pins: UninitWheelPins<I1, I2, O>) -> Self {
        let pins = pins.init();
        Self {
            pins,
            value: 0,
            state: true,
            prev_state: true,
            scroll_val: 0,
        }
    }
}

pub trait Scroller {
    fn read_scroll(&mut self) -> Option<KeyCode>;
}
impl<
        I1: InputPin<Error = Infallible>,
        I2: InputPin<Error = Infallible>,
        O: OutputPin<Error = Infallible>,
    > Scroller for MouseWheelDriver<I1, I2, O>
{
    fn read_scroll(&mut self) -> Option<KeyCode> {
        self.state = self.pins.in1.is_high().unwrap();
        let res = if self.state == self.prev_state {
            None
        } else {
            let scroll = if self.pins.in2.is_high().unwrap() == self.state {
                self.value -= 1;
                self.scroll_val = -1;
                KeyCode::MediaScrollDown
            } else {
                self.value += 1;
                self.scroll_val = 1;
                KeyCode::MediaScrollUp
            };
            Some(scroll)
        };
        self.prev_state = self.state;
        res
    }
}
