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
        is_shifted_r: bool,
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

#[derive(Debug, Clone, Copy)]
pub enum ShiftType {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

#[derive(Debug, Clone, Copy)]
pub enum DataProcOpcode {
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
    Eq_,
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
