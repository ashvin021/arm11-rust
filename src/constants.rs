pub const MEMORY_SIZE: usize = 65536;
pub const NUM_REGS: usize = 17;
pub const NUM_GENERAL_REGS: usize = 13;
pub const BYTES_IN_WORD: usize = 4;
pub const PIPELINE_OFFSET: usize = 8;

// Special Registers
pub const PC: usize = 15;
pub const CPSR: usize = 16;

// Instruction Fields

pub struct InstructionField {
    pub size: u8,
    pub pos: u32,
}

impl InstructionField {
    pub const fn new(size: u8, pos: u32) -> Self {
        InstructionField { size, pos }
    }

    pub const fn bit(pos: u32) -> Self {
        InstructionField { size: 1, pos }
    }
}

// Common instruction fields
pub const COND: InstructionField = InstructionField::new(4, 28);
pub const I: InstructionField = InstructionField::bit(25);
pub const S: InstructionField = InstructionField::bit(20);
pub const RN: InstructionField = InstructionField::new(4, 16);
pub const RD: InstructionField = InstructionField::new(4, 12);

// Processing instruction fields
pub const OPCODE: InstructionField = InstructionField::new(4, 21);

// Transfer instruction fields
pub const P: InstructionField = InstructionField::bit(24);
pub const U: InstructionField = InstructionField::bit(23);
pub const L: InstructionField = InstructionField::bit(20);

// Multiply instruction fields
pub const A: InstructionField = InstructionField::bit(21);
pub const RD_MULT: InstructionField = InstructionField::new(4, 16);
pub const RN_MULT: InstructionField = InstructionField::new(4, 12);
pub const RS: InstructionField = InstructionField::new(4, 8);
pub const RM: InstructionField = InstructionField::new(4, 0);

// Branch instruction fields
pub const OFFSET_BRANCH: InstructionField = InstructionField::new(24, 0);

// Operand2 / Offset sub-fields
pub const IMM_VALUE: InstructionField = InstructionField::new(8, 0);
pub const IMM_SHIFT: InstructionField = InstructionField::new(4, 8);
pub const SHIFT_TYPE: InstructionField = InstructionField::new(2, 5);
pub const CONST_SHIFT: InstructionField = InstructionField::new(5, 7);
pub const REG_SHIFT: InstructionField = InstructionField::new(4, 8);

// Bitmasking
pub const fn mask(size: u8) -> u32 {
    (1 << size) - 1
}
