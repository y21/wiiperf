mod opcodes {
    pub const B: u32 = 0b010010;
    pub const STWU: u32 = 0b100101;
}

pub fn branch(offset: isize, link: bool, absolute: bool) -> u32 {
    assert!(offset >= -8388608 && offset <= 8388607);

    let mut number = (offset >> 2) as u32 & 0x00FF_FFFF;

    // Space for AA and LK
    number <<= 2;
    number |= (absolute as u32) << 1;
    number |= link as u32;

    (opcodes::B << 26) | number
}

pub enum Instruction {
    Stwu { source: u8, dest: u8 },
}

/// Mini instruction decoder for things we need
pub fn decode_instr(instr: u32) -> Option<Instruction> {
    let op = instr >> 26;
    match op {
        opcodes::STWU => {
            let source = (instr >> 21) & 0x1f; // 6-10
            let dest = (instr >> 16) & 0x1f; // 11-15
            let _imm = instr & 0xffff; // 16-31
            Some(Instruction::Stwu {
                source: source as u8,
                dest: dest as u8,
            })
        }
        _ => None,
    }
}
