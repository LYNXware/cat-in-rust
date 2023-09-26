use keyberon::key_code::KeyCode;
use keyberon::{action::k, layout::Layers};

#[rustfmt::skip]
pub static LAYERS: Layers<6, 4, 1> = {
#[allow(clippy::enum_glob_use)]
use KeyCode::*;
[[
    [k(X),  k(Escape), k(LCtrl), k(LAlt), k(No), k(No)],
    [k(No), k(Y),      k(U),     k(I),    k(O),  k(P)],
    [k(No), k(H),      k(J),     k(K),    k(L),  k(M)],
    [k(No), k(X),      k(C),     k(V),    k(B),  k(R)],
]] 
};
