pub trait ToU32 {
    fn u32(self) -> u32;
}

pub trait ToUsize {
    fn usize(self) -> usize;
}

impl ToU32 for usize {
    fn u32(self) -> u32 {
        self as u32
    }
}

impl ToUsize for u32 {
    fn usize(self) -> usize {
        self as usize
    }
}
