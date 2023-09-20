//
use keyberon::key_code::KeyCode;
use keyberon::{action::k, layout::Layers};

#[rustfmt::skip]
pub static LAYERS: Layers<3, 4, 1> = {
#[allow(clippy::enum_glob_use)]
use KeyCode::*;
[[
    [k(Kb2), k(Kb3), k(Kb4)],
    [k(Kb1), k(C),  k(Kb5)],
    [k(B),  k(No),  k(D)],
    [k(A),  k(X),  k(E)],
]] 
};
