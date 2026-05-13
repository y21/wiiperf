mod opcodes {
    pub const B: u32 = 0b010010 << 26;
}

pub fn branch(offset: isize, link: bool, absolute: bool) -> u32 {
    assert!(offset >= -8388608 && offset <= 8388607);

    let mut number = (offset >> 2) as u32 & 0x00FF_FFFF;

    // Space for AA and LK
    number <<= 2;
    number |= (absolute as u32) << 1;
    number |= link as u32;

    opcodes::B | number
}
