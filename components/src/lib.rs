#![no_std]

use generic_array::{GenericArray, ArrayLength};
pub mod matrix;
pub mod mouse;

pub trait ReadState {
    type LEN: ArrayLength;
    fn read_state(&mut self, buf: &mut GenericArray<u8, Self::LEN>);
}
