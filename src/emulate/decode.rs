use nom::{
    bits,
    bits::complete::{tag, take},
    branch::alt,
    combinator::{map, map_opt, peek},
    sequence::{pair, preceded, terminated, tuple},
};

use num_traits::FromPrimitive;

use crate::{parse::*, types::*};

pub fn decode(instr: &u32) -> Result<ConditionalInstruction> {
    Ok(decode_conditional_instruction(&instr.to_be_bytes()[..])
        .map_err(|e| format!("{:#?}", e))?
        .1)
}

fn decode_conditional_instruction(input: &[u8]) -> NomResult<&[u8], ConditionalInstruction> {
    bits(map(
        tuple((
            decode_cond,
            alt((
                decode_halt,
                decode_multiply,
                decode_processing,
                decode_transfer,
                decode_branch,
            )),
        )),
        |(cond, instruction)| ConditionalInstruction { instruction, cond },
    ))(input)
}

fn decode_halt(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    map(tag(0, 28u32), |_| Instruction::Halt)(input)
}

fn decode_processing(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    let is_immediate = peek(preceded(take::<_, u32, _, _>(2u32), take_bool))(input)?.1;
    map(
        tuple((
            tag(0, 2u8),
            take_bool,
            decode_opcode,
            take_bool,
            take(4u8),
            take(4u8),
            if is_immediate {
                decode_operand2_immediate
            } else {
                decode_operand2_shifted
            },
        )),
        |(_, _, opcode, set_cond, rn, rd, operand2)| {
            Instruction::Processing(InstructionProcessing {
                opcode,
                set_cond,
                rn,
                rd,
                operand2,
            })
        },
    )(input)
}

fn decode_transfer(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    // Check if its an immediate or shifted register transfer
    let is_shifted_r = peek(preceded(take::<_, u32, _, _>(2u32), take_bool))(input)?.1;
    map(
        tuple((
            tag(1, 2u8),
            take_bool,
            take_bool,
            take_bool,
            tag(0, 2u8),
            take_bool,
            take(4u8),
            take(4u8),
            if is_shifted_r {
                decode_operand2_shifted
            } else {
                decode_operand2_immediate
            },
        )),
        |(_, _, is_preindexed, up_bit, _, load, rn, rd, offset)| {
            Instruction::Transfer(InstructionTransfer {
                load,
                up_bit,
                is_preindexed,
                rn,
                rd,
                offset,
            })
        },
    )(input)
}

fn decode_multiply(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    map(
        tuple((
            tag(0, 6u8),
            take_bool,
            take_bool,
            take(4u8),
            take(4u8),
            take(4u8),
            tag(0x9, 4u8),
            take(4u8),
        )),
        |(_, accumulate, set_cond, rd, rn, rs, _, rm)| {
            Instruction::Multiply(InstructionMultiply {
                accumulate,
                set_cond,
                rd,
                rn,
                rs,
                rm,
            })
        },
    )(input)
}

fn decode_branch(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    map(tuple((tag(0xa, 4u8), take(24u32))), |(_, offset)| {
        Instruction::Branch(InstructionBranch { offset })
    })(input)
}

fn take_bool(input: (&[u8], usize)) -> NomResult<(&[u8], usize), bool> {
    map(take(1u8), |i: u8| i == 1)(input)
}

fn decode_opcode(input: (&[u8], usize)) -> NomResult<(&[u8], usize), ProcessingOpcode> {
    map_opt(take(4u8), ProcessingOpcode::from_u8)(input)
}

fn decode_shift_type(input: (&[u8], usize)) -> NomResult<(&[u8], usize), ShiftType> {
    map_opt(take(2u8), ShiftType::from_u8)(input)
}

fn decode_cond(input: (&[u8], usize)) -> NomResult<(&[u8], usize), ConditionCode> {
    map_opt(take(4u8), ConditionCode::from_u8)(input)
}

fn decode_operand2_immediate(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Operand2> {
    map(tuple((take(4u8), take(8u8))), |(shift_amt, to_shift)| {
        Operand2::ConstantShift(to_shift, shift_amt)
    })(input)
}

fn decode_operand2_shifted(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Operand2> {
    // Check if its an constant shifted register or a shifted register
    let is_shifted_r = peek(preceded(take::<_, u8, _, _>(7u8), take_bool))(input)?.1;
    map(
        tuple((
            alt((
                pair(
                    terminated(take::<_, u8, _, _>(4u8), tag(0, 1u8)),
                    terminated(decode_shift_type, tag(1, 1u8)),
                ),
                pair(take(5u8), terminated(decode_shift_type, tag(0, 1u8))),
            )),
            take(4u8),
        )),
        move |((shift_amt, shift_type), reg_to_shift)| {
            if is_shifted_r {
                Operand2::ShiftedReg(reg_to_shift, Shift::RegisterShift(shift_type, shift_amt))
            } else {
                Operand2::ShiftedReg(reg_to_shift, Shift::ConstantShift(shift_type, shift_amt))
            }
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_operand2_immediate() {
        let bytes = 0x12a0u16.to_be_bytes();
        assert_eq!(
            bits(decode_operand2_immediate)(&bytes[..])
                .expect("operand2 decode failed")
                .1,
            Operand2::ConstantShift(0x2a, 0x1)
        );
    }

    #[test]
    fn test_decode_operand2_shifted() {
        let bytes = 0x12a0u16.to_be_bytes();
        assert_eq!(
            bits(decode_operand2_shifted)(&bytes[..])
                .expect("operand2 decode failed")
                .1,
            Operand2::ShiftedReg(0xa, Shift::ConstantShift(ShiftType::Lsr, 0x2))
        );
    }

    #[test]
    fn test_decode_halt() {
        let bytes = 0u32.to_be_bytes();
        assert_eq!(
            bits(decode_halt)(&bytes[..]).expect("parse halt failed").1,
            Instruction::Halt
        );
        assert_eq!(
            decode_conditional_instruction(&bytes[..])
                .expect("decode conditional halt failed")
                .1
                .instruction,
            Instruction::Halt
        );
    }

    #[test]
    fn test_decode_processing() {
        let bytes = 0xe3a01001u32.to_be_bytes();
        let expected = ConditionalInstruction {
            instruction: Instruction::Processing(InstructionProcessing {
                opcode: ProcessingOpcode::Mov,
                set_cond: false,
                rn: 0x0,
                rd: 0x1,
                operand2: Operand2::ConstantShift(0x1, 0x0),
            }),
            cond: ConditionCode::Al,
        };

        assert_eq!(
            decode_conditional_instruction(&bytes[..])
                .expect("decode conditional processing failed")
                .1,
            expected
        );
    }

    #[test]
    fn test_decode_multiply() {
        let bytes = 0xe0231290u32.to_be_bytes();
        let expected = ConditionalInstruction {
            instruction: Instruction::Multiply(InstructionMultiply {
                accumulate: true,
                set_cond: false,
                rd: 0x3,
                rn: 0x1,
                rs: 0x2,
                rm: 0x0,
            }),
            cond: ConditionCode::Al,
        };

        assert_eq!(
            decode_conditional_instruction(&bytes[..])
                .expect("decode conditional multiply failed")
                .1,
            expected
        );
    }

    #[test]
    fn test_decode_transfer() {
        let bytes = 0xe7196103u32.to_be_bytes();
        let expected = ConditionalInstruction {
            instruction: Instruction::Transfer(InstructionTransfer {
                is_preindexed: true,
                up_bit: false,
                load: true,
                rn: 9,
                rd: 6,
                offset: Operand2::ShiftedReg(3, Shift::ConstantShift(ShiftType::Lsl, 2)),
            }),
            cond: ConditionCode::Al,
        };

        assert_eq!(
            decode_conditional_instruction(&bytes[..])
                .expect("decode conditional transfer failed")
                .1,
            expected
        );
    }

    #[test]
    fn test_decode_branch() {
        let bytes = 0x0a000121u32.to_be_bytes();
        let expected = ConditionalInstruction {
            instruction: Instruction::Branch(InstructionBranch { offset: 0x000121 }),
            cond: ConditionCode::Eq,
        };

        assert_eq!(
            decode_conditional_instruction(&bytes[..])
                .expect("decode conditional branch failed")
                .1,
            expected
        );
    }
}
