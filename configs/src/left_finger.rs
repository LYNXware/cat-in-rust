use keyberon::key_code::KeyCode;
use keyberon::{action::k, layout::Layers};

#[rustfmt::skip]
pub static LAYERS: Layers<6, 4, 1> = {
#[allow(clippy::enum_glob_use)]
use KeyCode::*;
[[
    [k(No), k(No), k(Escape), k(LCtrl), k(LAlt), k(No)],
    [k(No), k(Q),  k(W),      k(E),     k(R),    k(T) ],
    [k(Q),  k(A),  k(S),      k(D),     k(F),    k(G) ],
    [k(No), k(Z),  k(X),      k(C),     k(V),    k(B) ],
]] 
};
