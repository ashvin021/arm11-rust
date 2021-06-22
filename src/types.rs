use std::{error, result};

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy)]
pub struct InstructionProcessing {
    cond: ConditionCode,
    opcode: ProcessingOpcode,
    set_cond: bool,
    rn: u8,
    rd: u8,
    operand2: Operand2,
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionMultiply {
    accumulate: bool,
    set_cond: bool,
    rd: u8,
    rn: u8,
    rs: u8,
    rm: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionTransfer {
    is_shifted_r: bool,
    is_preindexed: bool,
    up_bit: bool,
    load: bool,
    rn: u8,
    rd: u8,
    offset: Operand2,
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionBranch {
    cond: ConditionCode,
    offset: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Processing(InstructionProcessing),
    Multiply(InstructionMultiply),
    Branch(InstructionBranch),
    Transfer(InstructionTransfer),
    Halt,
}

#[derive(Debug, Clone, Copy)]
pub struct ConditionalInstruction {
    pub instruction: Instruction,
    pub cond: ConditionCode,
}

#[derive(Debug, Clone, Copy)]
pub enum Operand2 {
    ConstantShift(u8, u8),
    ConstantShiftedReg(u8, ShiftType, u8),
    ShiftedReg(u8, ShiftType, u8),
}

#[derive(Debug, Clone, Copy)]
pub enum ShiftType {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessingOpcode {
    And,
    Eor,
    Sub,
    Rsb,
    Add,
    Cmp,
    Tst,
    Teq,
    Orr,
    Mov,
}

#[derive(Debug, Clone, Copy)]
pub enum ConditionCode {
    Eq,
    Ne,
    Ge,
    Lt,
    Gt,
    Le,
    Al,
}

pub enum CpsrFlag {
    VFlag = 28,
    CFlag = 29,
    ZFlag = 30,
    NFlag = 31,
}
