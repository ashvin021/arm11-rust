use super::utils::*;
use crate::types::*;

pub fn satisfies_cpsr(cond: &ConditionCode, cpsr_contents: &u32) -> bool {
    let n: bool = extract_bit(cpsr_contents, 31);
    let z: bool = extract_bit(cpsr_contents, 30);
    let v: bool = extract_bit(cpsr_contents, 28);

    match cond {
        ConditionCode::Eq_ => z,
        ConditionCode::Ne => !z,
        ConditionCode::Ge => n == v,
        ConditionCode::Lt => n != v,
        ConditionCode::Gt => !z && (n == v),
        ConditionCode::Le => z || (n != v),
        ConditionCode::Al => true,
    }
}

pub fn set_flags(flag: CpsrFlag, set: bool, cpsr_contents: &mut u32) {
    if set {
        *cpsr_contents |= 1 << flag as u32;
    } else {
        *cpsr_contents &= !1 << flag as u32;
    }
}

pub fn barrel_shifter(op2: Operand2, register_file: &[u32; 17]) -> (u32, bool) {
    let (shift_amt, to_shift, shift_type): (u8, u32, ShiftType) = match op2 {
        Operand2::ConstantShift(shift_amt, to_shift) => {
            (shift_amt, to_shift as u32, ShiftType::Ror)
        }
        Operand2::ConstantShiftedReg(constant_shift, shift_type, reg_to_shift) => (
            constant_shift,
            register_file[reg_to_shift as usize],
            shift_type,
        ),
        Operand2::ShiftedReg(shift_reg, shift_type, reg_to_shift) => (
            (register_file[shift_reg as usize] & mask(8)) as u8,
            register_file[reg_to_shift as usize],
            shift_type,
        ),
    };

    shift(to_shift, shift_amt, shift_type)
}

pub fn shift(to_shift: u32, shift_amt: u8, shift_type: ShiftType) -> (u32, bool) {
    match shift_type {
        ShiftType::Lsl => to_shift.overflowing_shl(shift_amt as u32),
        ShiftType::Lsr => to_shift.overflowing_shr(shift_amt as u32),
        ShiftType::Asr => {
            let (res, cout) = (to_shift as i32).overflowing_shr(shift_amt as u32);
            (res as u32, cout)
        }
        ShiftType::Ror => (
            to_shift.rotate_right(shift_amt as u32),
            extract_bit(&to_shift, shift_amt - 1),
        ),
    }
}

pub fn perform_dataproc_operation(op1: i32, op2: i32, opcode: DataProcOpcode) -> (i32, bool) {
    match opcode {
        DataProcOpcode::And | DataProcOpcode::Tst => (op1 & op2, false),
        DataProcOpcode::Eor | DataProcOpcode::Teq => (op1 ^ op2, false),
        DataProcOpcode::Sub => op1.overflowing_sub(op2),
        DataProcOpcode::Rsb => op2.overflowing_sub(op1),
        DataProcOpcode::Add => op1.overflowing_add(op1),
        DataProcOpcode::Cmp => (op1 - op2, !(op1 < op2)),
        DataProcOpcode::Orr => (op1 | op2, false),
        DataProcOpcode::Mov => (op2, false),
    }
}
