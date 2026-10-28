#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn is_plausible(&self) -> bool {
        (64..=16384).contains(&self.width) && (64..=16384).contains(&self.height)
    }
}
