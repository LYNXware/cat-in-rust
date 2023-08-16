use core::convert::Infallible;

use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_backtrace as _;
use keyberon::key_code::KeyCode;

pub struct UninitWheelPins<'a, I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    pub pin_a: &'a I1,
    pub pin_b: &'a I2,
    pub gnd: &'a mut O,
}

impl<'a, I1, I2, O> UninitWheelPins<'a, I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    fn init(self) -> InitedWheelPins<'a, I1, I2, O> {
        let _ = self.gnd.set_low();
        InitedWheelPins {
            pin_a: self.pin_a,
            pin_b: self.pin_b,
            _gnd: self.gnd,
        }
    }
}

pub struct InitedWheelPins<'a, I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    pin_a: &'a I1,
    pin_b: &'a I2,
    _gnd: &'a O,
}

pub struct MouseWheelDriver<'a, I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    pins: InitedWheelPins<'a, I1, I2, O>,
    value: u8,
    state: bool,
    prev_state: bool,
    scroll_val: i8,
}

impl<'a, I1, I2, O> MouseWheelDriver<'a, I1, I2, O>
where
    I1: InputPin<Error = Infallible>,
    I2: InputPin<Error = Infallible>,
    O: OutputPin<Error = Infallible>,
{
    pub fn new(pins: UninitWheelPins<'a, I1, I2, O>) -> Self {
        let pins = pins.init();
        Self {
            pins,
            value: 0,
            state: true,
            prev_state: true,
            scroll_val: 0,
        }
    }
    pub fn read_encoder(&mut self) -> Option<KeyCode> {
        self.state = self.pins.pin_a.is_high().unwrap();
        let res = if self.state == self.prev_state {
            None
        } else {
            let scroll = if self.pins.pin_b.is_high().unwrap() == self.state {
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
