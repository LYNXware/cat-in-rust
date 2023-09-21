pub struct DyDx {
    pub dy: u8,
    pub dx: u8,
}

pub trait PointerRead {
    fn pointer_read(&mut self) -> DyDx;
}
