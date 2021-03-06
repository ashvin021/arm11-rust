use nom::{
    bits,
    bits::complete::{tag, take},
    branch::alt,
    combinator::{map, map_opt, peek},
    error::context,
    sequence::{pair, preceded, terminated, tuple},
};

use num_traits::FromPrimitive;

use crate::{constants::*, parse::*, types::*};

pub fn decode(instr: &u32) -> Result<ConditionalInstruction> {
    // A zero instruction is Halt
    if *instr == 0 {
        return Ok(ConditionalInstruction {
            cond: ConditionCode::Eq,
            instruction: Instruction::Halt,
        });
    }

    let mut decoder = bits(decode_conditional_instruction);
    Ok(decoder(&instr.to_be_bytes())
        .map_err(|e| format!("{:#?}", e))?
        .1)
}

fn decode_conditional_instruction(
    input: (&[u8], usize),
) -> NomResult<(&[u8], usize), ConditionalInstruction> {
    let instr_type: (u32, u32) = context(
        "peeking conditional instruction type",
        peek(tuple((
            preceded(take::<_, u32, _, _>(4u32), take(2u32)),
            preceded(take::<_, u32, _, _>(18u32), take(4u32)),
        ))),
    )(input)?
    .1;

    let decode_instr = match instr_type {
        (0x0, 0x9) => decode_multiply,
        (0x0, _) => decode_processing,
        (0x1, _) => decode_transfer,
        (0x2, _) => decode_branch,
        _ => return Err(ArmNomError::new(ArmNomErrorKind::InvalidInstructionType).into()),
    };

    context(
        "decoding conditional instruction",
        map(tuple((decode_cond, decode_instr)), |(cond, instruction)| {
            ConditionalInstruction { instruction, cond }
        }),
    )(input)
}

fn decode_processing(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    let is_immediate = peek(preceded(take::<_, u32, _, _>(2u32), take_bool))(input)?.1;
    context(
        "decoding processing instruction",
        map(
            tuple((
                tag(0, 2u8),
                take_bool,
                decode_opcode,
                take_bool,
                take(RN.size),
                take(RD.size),
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
        ),
    )(input)
}

fn decode_transfer(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    // Check if its an immediate or shifted register transfer
    let is_shifted_r = peek(preceded(take::<_, u32, _, _>(2u32), take_bool))(input)?.1;
    context(
        "decoding transfer instruction",
        map(
            tuple((
                tag(1, 2u8),
                take_bool,
                take_bool,
                take_bool,
                tag(0, 2u8),
                take_bool,
                take(RN.size),
                take(RD.size),
                if is_shifted_r {
                    decode_operand2_shifted
                } else {
                    decode_operand2_immediate
                },
            )),
            |(_, _, is_preindexed, up_bit, _, load, rn, rd, offset)| {
                Instruction::Transfer(InstructionTransfer {
                    is_preindexed,
                    up_bit,
                    load,
                    rn,
                    rd,
                    offset,
                })
            },
        ),
    )(input)
}

fn decode_multiply(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    context(
        "decoding multiply instruction",
        map(
            tuple((
                tag(0, 6u8),
                take_bool,
                take_bool,
                take(RD_MULT.size),
                take(RN_MULT.size),
                take(RS.size),
                tag(0x9, 4u8),
                take(RM.size),
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
        ),
    )(input)
}

fn decode_branch(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Instruction> {
    context(
        "decoding branch instruction",
        map(
            tuple((tag(0xa, 4u8), take(OFFSET_BRANCH.size))),
            |(_, offset)| Instruction::Branch(InstructionBranch { offset }),
        ),
    )(input)
}

fn take_bool(input: (&[u8], usize)) -> NomResult<(&[u8], usize), bool> {
    map(take(1u8), |i: u8| i == 1)(input)
}

fn decode_opcode(input: (&[u8], usize)) -> NomResult<(&[u8], usize), ProcessingOpcode> {
    context(
        "decoding processing opcode",
        map_opt(take(OPCODE.size), ProcessingOpcode::from_u8),
    )(input)
}

fn decode_shift_type(input: (&[u8], usize)) -> NomResult<(&[u8], usize), ShiftType> {
    context(
        "decoding shift type",
        map_opt(take(SHIFT_TYPE.size), ShiftType::from_u8),
    )(input)
}

fn decode_cond(input: (&[u8], usize)) -> NomResult<(&[u8], usize), ConditionCode> {
    context(
        "decoding condition code",
        map_opt(take(COND.size), ConditionCode::from_u8),
    )(input)
}

fn decode_operand2_immediate(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Operand2> {
    context(
        "decoding operand2 immediate",
        map(
            tuple((take(IMM_SHIFT.size), take(IMM_VALUE.size))),
            |(shift_amt, to_shift)| Operand2::ConstantShift(to_shift, shift_amt),
        ),
    )(input)
}

fn decode_operand2_shifted(input: (&[u8], usize)) -> NomResult<(&[u8], usize), Operand2> {
    // Check if its an constant shifted register or a shifted register
    let is_shifted_r = peek(preceded(take::<_, u8, _, _>(7u8), take_bool))(input)?.1;
    context(
        "decoding operand2 shifted",
        map(
            tuple((
                alt((
                    pair(
                        terminated(take::<_, u8, _, _>(REG_SHIFT.size), tag(0, 1u8)),
                        terminated(decode_shift_type, tag(1, 1u8)),
                    ),
                    pair(
                        take(CONST_SHIFT.size),
                        terminated(decode_shift_type, tag(0, 1u8)),
                    ),
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
        ),
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
        assert_eq!(
            decode(&0u32).expect("parse halt failed").instruction,
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
            bits(decode_conditional_instruction)(&bytes[..])
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
            bits(decode_conditional_instruction)(&bytes[..])
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
            bits(decode_conditional_instruction)(&bytes[..])
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
            bits(decode_conditional_instruction)(&bytes[..])
                .expect("decode conditional branch failed")
                .1,
            expected
        );
    }
}
