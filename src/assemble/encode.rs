use crate::types::*;

pub fn encode(instr: ConditionalInstruction) -> u32 {
    let cond = (instr.cond as u32).rotate_right(4);
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

    (is_immediate as u32) << 25
        | (opcode as u32) << 21
        | (set_cond as u32) << 20
        | u32::from(rn) << 16
        | u32::from(rd) << 12
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

    let mask = 0x90;

    mask | (accumulate as u32) << 21
        | (set_cond as u32) << 20
        | u32::from(rd) << 16
        | u32::from(rn) << 12
        | u32::from(rs) << 8
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

    let mask = 0x1 << 26;
    mask | (is_shifted_r as u32) << 25
        | (is_preindexed as u32) << 24
        | (up_bit as u32) << 23
        | (load as u32) << 20
        | u32::from(rn) << 16
        | u32::from(rd) << 12
        | encode_operand2(offset)
}

fn encode_branch(instr: InstructionBranch) -> u32 {
    let InstructionBranch { offset } = instr;
    let mask = 0x5 << 25;
    mask | (offset as u32)
}

fn encode_operand2(op2: Operand2) -> u32 {
    match op2 {
        Operand2::ConstantShift(to_shift, shift_amt) => {
            u32::from(shift_amt) << 8 | u32::from(to_shift)
        }
        Operand2::ShiftedReg(reg_to_shift, Shift::ConstantShift(shift_type, constant_shift)) => {
            u32::from(constant_shift) << 7 | (shift_type as u32) << 5 | u32::from(reg_to_shift)
        }
        Operand2::ShiftedReg(reg_to_shift, Shift::RegisterShift(shift_type, shift_reg)) => {
            u32::from(shift_reg) << 8 | (shift_type as u32) << 5 | 1 << 4 | u32::from(reg_to_shift)
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
