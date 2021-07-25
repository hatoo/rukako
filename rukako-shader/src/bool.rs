#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Bool32(u32);

impl Bool32 {
    pub const TRUE: Self = Self(1);
    pub const FALSE: Self = Self(0);
}

impl Bool32 {
    pub fn new(b: bool) -> Self {
        if b {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }
}

impl Into<bool> for Bool32 {
    fn into(self) -> bool {
        self == Self::TRUE
    }
}

impl From<bool> for Bool32 {
    fn from(b: bool) -> Self {
        if b {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }
}
