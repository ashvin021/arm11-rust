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
    Ok(parse_conditional_instruction(&instr.to_be_bytes()[..])
        .map_err(|e| format!("{:#?}", e))?
        .1)
}

fn parse_conditional_instruction<'a>(
    input: &'a [u8],
) -> NomResult<&'a [u8], ConditionalInstruction> {
    bits(map(
        tuple((
            parse_cond,
            alt((
                parse_halt,
                parse_multiply,
                parse_processing,
                parse_transfer,
                parse_branch,
            )),
        )),
        |(cond, instruction)| ConditionalInstruction { instruction, cond },
    ))(input)
}

fn parse_halt(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    map(tag(0, 28u32), |_| Instruction::Halt)(input)
}

fn parse_processing(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    let is_immediate = peek(preceded(take::<_, u32, _, _>(2u32), take_bool))(input)?.1;
    map(
        tuple((
            tag(0, 2u8),
            take_bool,
            parse_opcode,
            take_bool,
            take(4u8),
            take(4u8),
            if is_immediate {
                parse_operand2_immediate
            } else {
                parse_operand2_shifted
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

fn parse_transfer(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
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
                parse_operand2_shifted
            } else {
                parse_operand2_immediate
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

fn parse_multiply(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
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

fn parse_branch(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    map(tuple((tag(0xa, 4u8), take(24u32))), |(_, offset)| {
        Instruction::Branch(InstructionBranch { offset })
    })(input)
}

fn take_bool(input: (&[u8], usize)) -> NomResult<(&[u8], usize), bool> {
    map(take(1u8), |i: u8| i == 1)(input)
}

fn parse_opcode(input: (&[u8], usize)) -> NomResult<(&[u8], usize), ProcessingOpcode> {
    map_opt(take(4u8), |opcode: u8| ProcessingOpcode::from_u8(opcode))(input)
}

fn parse_shift_type(input: (&[u8], usize)) -> NomResult<(&[u8], usize), ShiftType> {
    map_opt(take(2u8), |shift: u8| ShiftType::from_u8(shift))(input)
}

fn parse_cond(input: (&[u8], usize)) -> NomResult<(&[u8], usize), ConditionCode> {
    map_opt(take(4u8), |cond: u8| ConditionCode::from_u8(cond))(input)
}

fn parse_operand2_immediate(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Operand2> {
    map(tuple((take(4u8), take(8u8))), |(shift_amt, to_shift)| {
        Operand2::ConstantShift(shift_amt, to_shift)
    })(input)
}

fn parse_operand2_shifted(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Operand2> {
    let is_shifted_r = peek(preceded(take::<_, u8, _, _>(7u8), take_bool))(input)?.1;
    map(
        tuple((
            alt((
                pair(
                    terminated(take::<_, u8, _, _>(4u8), tag(0, 1u8)),
                    terminated(parse_shift_type, tag(1, 1u8)),
                ),
                pair(take(5u8), terminated(parse_shift_type, tag(0, 1u8))),
            )),
            take(4u8),
        )),
        move |((shift_amt, shift_type), reg_to_shift)| {
            if is_shifted_r {
                Operand2::ShiftedReg(shift_amt, shift_type, reg_to_shift)
            } else {
                Operand2::ConstantShiftedReg(shift_amt, shift_type, reg_to_shift)
            }
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_operand2_immediate() {
        let bytes = 0x12a0u16.to_be_bytes();
        assert_eq!(
            bits(parse_operand2_immediate)(&bytes[..])
                .expect("operand2 parse failed")
                .1,
            Operand2::ConstantShift(0x1, 0x2a)
        );
    }

    #[test]
    fn test_parse_operand2_shifted() {
        let bytes = 0x12a0u16.to_be_bytes();
        assert_eq!(
            bits(parse_operand2_shifted)(&bytes[..])
                .expect("operand2 parse failed")
                .1,
            Operand2::ConstantShiftedReg(0x2, ShiftType::Lsr, 0xa)
        );
    }

    #[test]
    fn test_parse_halt() {
        let bytes = 0u32.to_be_bytes();
        assert_eq!(
            bits(parse_halt)(&bytes[..]).expect("parse halt failed").1,
            Instruction::Halt
        );
        assert_eq!(
            parse_conditional_instruction(&bytes[..])
                .expect("parse conditional halt failed")
                .1
                .instruction,
            Instruction::Halt
        );
    }

    #[test]
    fn test_parse_processing() {
        let bytes = 0xe3a01001u32.to_be_bytes();
        let expected = ConditionalInstruction {
            instruction: Instruction::Processing(InstructionProcessing {
                opcode: ProcessingOpcode::Mov,
                set_cond: false,
                rn: 0x0,
                rd: 0x1,
                operand2: Operand2::ConstantShift(0x0, 0x1),
            }),
            cond: ConditionCode::Al,
        };

        assert_eq!(
            parse_conditional_instruction(&bytes[..])
                .expect("parse conditional processing failed")
                .1,
            expected
        );
    }

    #[test]
    fn test_parse_multiply() {
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
            parse_conditional_instruction(&bytes[..])
                .expect("parse conditional multiply failed")
                .1,
            expected
        );
    }

    #[test]
    fn test_parse_transfer() {
        let bytes = 0xe7196103u32.to_be_bytes();
        let expected = ConditionalInstruction {
            instruction: Instruction::Transfer(InstructionTransfer {
                is_preindexed: true,
                up_bit: false,
                load: true,
                rn: 9,
                rd: 6,
                offset: Operand2::ConstantShiftedReg(2, ShiftType::Lsl, 3),
            }),
            cond: ConditionCode::Al,
        };

        assert_eq!(
            parse_conditional_instruction(&bytes[..])
                .expect("parse conditional transfer failed")
                .1,
            expected
        );
    }

    #[test]
    fn test_parse_branch() {
        let bytes = 0x0a000121u32.to_be_bytes();
        let expected = ConditionalInstruction {
            instruction: Instruction::Branch(InstructionBranch { offset: 0x000121 }),
            cond: ConditionCode::Eq,
        };

        assert_eq!(
            parse_conditional_instruction(&bytes[..])
                .expect("parse conditional branch failed")
                .1,
            expected
        );
    }
}
