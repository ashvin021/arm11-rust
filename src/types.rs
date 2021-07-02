use enum_primitive_derive::Primitive;
use std::{error, result};

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstructionProcessing {
    pub opcode: ProcessingOpcode,
    pub set_cond: bool,
    pub rn: u8,
    pub rd: u8,
    pub operand2: Operand2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstructionMultiply {
    pub accumulate: bool,
    pub set_cond: bool,
    pub rd: u8,
    pub rn: u8,
    pub rs: u8,
    pub rm: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstructionTransfer {
    pub is_preindexed: bool,
    pub up_bit: bool,
    pub load: bool,
    pub rn: u8,
    pub rd: u8,
    pub offset: Operand2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstructionBranch {
    pub offset: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Processing(InstructionProcessing),
    Multiply(InstructionMultiply),
    Branch(InstructionBranch),
    Transfer(InstructionTransfer),
    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConditionalInstruction {
    pub instruction: Instruction,
    pub cond: ConditionCode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand2 {
    ConstantShift(u8, u8),
    ConstantShiftedReg(u8, ShiftType, u8),
    ShiftedReg(u8, ShiftType, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Primitive)]
pub enum ShiftType {
    Lsl = 0x0,
    Lsr = 0x1,
    Asr = 0x2,
    Ror = 0x3,
}

#[derive(Debug, Clone, Copy, PartialEq, Primitive)]
pub enum ProcessingOpcode {
    And = 0x0,
    Eor = 0x1,
    Sub = 0x2,
    Rsb = 0x3,
    Add = 0x4,
    Tst = 0x8,
    Teq = 0x9,
    Cmp = 0xa,
    Orr = 0xc,
    Mov = 0xd,
}

#[derive(Debug, Clone, Copy, PartialEq, Primitive)]
pub enum ConditionCode {
    Eq = 0x0,
    Ne = 0x1,
    Ge = 0xa,
    Lt = 0xb,
    Gt = 0xc,
    Le = 0xd,
    Al = 0xe,
}

pub enum CpsrFlag {
    V = 28,
    C = 29,
    Z = 30,
    N = 31,
}
