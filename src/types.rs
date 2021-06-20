use enum_primitive::enum_from_primitive;
use std::{error, result};

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    DataProc {
        cond: ConditionCode,
        opcode: DataProcOpcode,
        set_cond: bool,
        rn: u8,
        rd: u8,
        operand2: Operand2,
    },

    Multiply {
        cond: ConditionCode,
        accumulate: bool,
        set_cond: bool,
        rd: u8,
        rn: u8,
        rs: u8,
        rm: u8,
    },

    SDT {
        cond: ConditionCode,
        is_preindexed: bool,
        up_bit: bool,
        load: bool,
        rn: u8,
        rd: u8,
        offset: Operand2,
    },

    Branch {
        cond: ConditionCode,
        offset: i32,
    },

    Raw(u32),
}

#[derive(Debug, Clone, Copy)]
pub enum Operand2 {
    ConstantShift(u8, u8),
    ConstantShiftedReg(u8, ShiftType, u8),
    ShiftedReg(u8, ShiftType, u8),
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum ShiftType {
    Lsl = 0x0,
    Lsr = 0x1,
    Asr = 0x2,
    Ror = 0x3,
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum DataProcOpcode {
    And = 0x0,
    Eor = 0x1,
    Sub = 0x2,
    Rsb = 0x3,
    Add = 0x4,
    Cmp = 0x8,
    Tst = 0x9,
    Teq = 0xa,
    Orr = 0xc,
    Mov = 0xd,
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum ConditionCode {
    Eq_ = 0x0,
    Ne = 0x1,
    Ge = 0xa,
    Lt = 0xb,
    Gt = 0xc,
    Le = 0xd,
    Al = 0xe,
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum CpsrFlag {
    VFlag = 28,
    CFlag = 29,
    ZFlag = 30,
    NFlag = 31,
}
}
