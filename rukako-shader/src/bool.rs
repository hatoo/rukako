#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bool32(u32);

impl Bool32 {
    pub const TRUE: Self = Self(1);
    pub const FALSE: Self = Self(0);
}
