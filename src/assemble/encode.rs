use crate::{constants::*, types::*};

pub fn encode(instr: ConditionalInstruction) -> u32 {
    let cond = (instr.cond as u32) << COND.pos;
    let body = match instr.instruction {
        Instruction::Processing(p) => encode_processing(p),
        Instruction::Transfer(t) => encode_transfer(t),
        Instruction::Multiply(m) => encode_multiply(m),
        Instruction::Branch(b) => encode_branch(b),
        Instruction::Halt => 0,
    };
    cond | body
}

fn encode_processing(instr: InstructionProcessing) -> u32 {
    let InstructionProcessing {
        opcode,
        set_cond,
        rn,
        rd,
        operand2,
    } = instr;

    let is_immediate = matches!(operand2, Operand2::ConstantShift(_, _));

    (is_immediate as u32) << I.pos
        | (opcode as u32) << OPCODE.pos
        | (set_cond as u32) << S.pos
        | u32::from(rn) << RN.pos
        | u32::from(rd) << RD.pos
        | encode_operand2(operand2)
}

fn encode_multiply(instr: InstructionMultiply) -> u32 {
    let InstructionMultiply {
        accumulate,
        set_cond,
        rd,
        rn,
        rs,
        rm,
    } = instr;

    // Constant base for all multiply instructions
    const BASE: u32 = 0x9 << 4;

    BASE | (accumulate as u32) << A.pos
        | (set_cond as u32) << S.pos
        | u32::from(rd) << RD_MULT.pos
        | u32::from(rn) << RN_MULT.pos
        | u32::from(rs) << RS.pos
        | u32::from(rm)
}

fn encode_transfer(instr: InstructionTransfer) -> u32 {
    let InstructionTransfer {
        is_preindexed,
        up_bit,
        load,
        rn,
        rd,
        offset,
    } = instr;

    let is_shifted_r = matches!(offset, Operand2::ShiftedReg(_, _));
    // Constant base for all transfer instructions
    const BASE: u32 = 0x1 << 26;

    BASE | (is_shifted_r as u32) << I.pos
        | (is_preindexed as u32) << P.pos
        | (up_bit as u32) << U.pos
        | (load as u32) << L.pos
        | u32::from(rn) << RN.pos
        | u32::from(rd) << RD.pos
        | encode_operand2(offset)
}

fn encode_branch(instr: InstructionBranch) -> u32 {
    let InstructionBranch { offset } = instr;
    // Constant base for all branch instructions
    const BASE: u32 = 0x5 << 25;
    BASE | ((offset as u32) & mask(OFFSET_BRANCH.size))
}

fn encode_operand2(op2: Operand2) -> u32 {
    match op2 {
        Operand2::ConstantShift(to_shift, shift_amt) => {
            u32::from(shift_amt) << IMM_SHIFT.pos | u32::from(to_shift)
        }
        Operand2::ShiftedReg(reg_to_shift, Shift::ConstantShift(shift_type, constant_shift)) => {
            u32::from(constant_shift) << CONST_SHIFT.pos
                | (shift_type as u32) << SHIFT_TYPE.pos
                | u32::from(reg_to_shift)
        }
        Operand2::ShiftedReg(reg_to_shift, Shift::RegisterShift(shift_type, shift_reg)) => {
            // Constant base for register shifted by register operand2 values
            const BASE: u32 = 1 << 4;
            BASE | u32::from(shift_reg) << REG_SHIFT.pos
                | (shift_type as u32) << SHIFT_TYPE.pos
                | u32::from(reg_to_shift)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_operand2() {
        assert_eq!(encode_operand2(Operand2::ConstantShift(0x8, 0x3)), 0x308);
        assert_eq!(
            encode_operand2(Operand2::ShiftedReg(
                0x7,
                Shift::ConstantShift(ShiftType::Ror, 0x3)
            )),
            0x1e7
        );
        assert_eq!(
            encode_operand2(Operand2::ShiftedReg(
                0x7,
                Shift::RegisterShift(ShiftType::Ror, 0x3)
            )),
            0x377
        );
    }
}
