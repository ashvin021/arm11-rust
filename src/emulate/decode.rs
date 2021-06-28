use nom::{
    bits,
    bits::complete::{tag, take},
    branch::alt,
    combinator::{map, map_opt, peek},
    sequence::{preceded, tuple},
    IResult,
};

use num_traits::FromPrimitive;

use super::utils;
use crate::types::*;

pub fn decode(instr: &u32) -> Result<ConditionalInstruction> {
    Ok(
        parse_conditional_instruction(&utils::to_u8_slice(*instr)[..])
            .map_err(|e| format!("{:#?}", e))?
            .1,
    )
}

fn parse_conditional_instruction<'a>(input: &'a [u8]) -> IResult<&'a [u8], ConditionalInstruction> {
    bits(map(
        tuple((
            alt((
                parse_processing,
                parse_transfer,
                parse_multiply,
                parse_branch,
            )),
            parse_cond,
        )),
        |(instruction, cond)| ConditionalInstruction { instruction, cond },
    ))(input)
}

fn parse_processing(input: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    let is_immediate = peek(preceded(take::<_, u32, _, _>(25u32), take_bool))(input)?.1;
    map(
        tuple((
            if is_immediate {
                parse_operand2_immediate
            } else {
                parse_operand2_shifted
            },
            take(4u8),
            take(4u8),
            take_bool,
            parse_opcode,
            take_bool,
            tag(0, 2u8),
        )),
        |(operand2, rd, rn, set_cond, opcode, _, _)| {
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

fn parse_transfer(input: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    let is_shifted_r = peek(preceded(take::<_, u32, _, _>(25u32), take_bool))(input)?.1;
    map(
        tuple((
            if !is_shifted_r {
                parse_operand2_immediate
            } else {
                parse_operand2_shifted
            },
            take(4u8),
            take(4u8),
            take_bool,
            tag(0, 2u8),
            take_bool,
            take_bool,
            take_bool,
            tag(1, 2u8),
        )),
        |(offset, rd, rn, load, _, up_bit, is_preindexed, _, _)| {
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

fn parse_multiply(input: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    map(
        tuple((
            take(4u8),
            tag(0x9, 4u8),
            take(4u8),
            take(4u8),
            take(4u8),
            take_bool,
            take_bool,
            tag(0, 6u8),
        )),
        |(rm, _, rs, rn, rd, set_cond, accumulate, _)| {
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

fn parse_branch(input: (&[u8], usize)) -> IResult<(&[u8], usize), Instruction> {
    map(tuple((take(24u32), tag(0xa, 4u8))), |(offset, _)| {
        Instruction::Branch(InstructionBranch { offset })
    })(input)
}

fn take_bool(input: (&[u8], usize)) -> IResult<(&[u8], usize), bool> {
    map(take(1u8), |i: u8| i == 1)(input)
}

fn parse_opcode(input: (&[u8], usize)) -> IResult<(&[u8], usize), ProcessingOpcode> {
    map_opt(take(4u8), |opcode: u8| ProcessingOpcode::from_u8(opcode))(input)
}

fn parse_shift_type(input: (&[u8], usize)) -> IResult<(&[u8], usize), ShiftType> {
    map_opt(take(4u8), |shift: u8| ShiftType::from_u8(shift))(input)
}

fn parse_cond(input: (&[u8], usize)) -> IResult<(&[u8], usize), ConditionCode> {
    map_opt(take(4u8), |cond: u8| ConditionCode::from_u8(cond))(input)
}

fn parse_operand2_immediate(input: (&[u8], usize)) -> IResult<(&[u8], usize), Operand2> {
    map(tuple((take(8u8), take(4u8))), |(to_shift, shift_amt)| {
        Operand2::ConstantShift(shift_amt, to_shift)
    })(input)
}

fn parse_operand2_shifted(input: (&[u8], usize)) -> IResult<(&[u8], usize), Operand2> {
    let (input, reg_to_shift) = take::<_, u8, _, _>(4u8)(input)?;
    let (input, is_constant_shifted) = take_bool(input)?;
    map(
        tuple((
            parse_shift_type,
            alt((preceded(take::<_, u8, _, _>(1u8), take(4u8)), take(5u8))),
        )),
        move |(shift_type, shift_amt)| {
            if is_constant_shifted {
                Operand2::ConstantShiftedReg(shift_amt, shift_type, reg_to_shift)
            } else {
                Operand2::ShiftedReg(shift_amt, shift_type, reg_to_shift)
            }
        },
    )(input)
}
