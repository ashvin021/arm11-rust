use std::{collections::HashMap, convert::TryInto, rc::Rc};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, digit1, hex_digit1},
    combinator::{map, map_opt, opt, recognize, success, value, verify},
    sequence::{delimited, preceded, terminated, tuple},
};

use crate::{constants::*, parse::*, types::*};

pub fn parse_asm(
    raw: &str,
    current_address: u32,
    next_free_address: u32,
    symbol_table: Rc<HashMap<String, u32>>,
) -> Result<(ConditionalInstruction, Option<u32>)> {
    let (instr, opt_data) = alt((
        parse_processing,
        parse_transfer(current_address, next_free_address),
        parse_multiply,
        parse_branch(current_address, symbol_table),
    ))(raw)
    .map_err(|e| format!("{:#?}", e))?
    .1;

    Ok((instr, opt_data))
}

fn parse_processing(input: &str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    map(
        tuple((
            terminated(parse_processing_opcode, char(' ')),
            terminated(parse_reg, char(',')),
            opt(terminated(parse_reg, char(','))),
            alt((parse_operand2_constant, parse_operand2_shifted)),
        )),
        |(opcode, r1, opt_r2, operand2)| {
            let (rd, rn, set_cond) = match (opcode, r1, opt_r2) {
                (_, rd, Some(rn)) => (rd, rn, false),
                (ProcessingOpcode::Mov, rd, None) => (rd, 0, false),
                (_, rn, None) => (0, rn, true),
            };

            (
                ConditionalInstruction {
                    cond: ConditionCode::Al,
                    instruction: Instruction::Processing(InstructionProcessing {
                        opcode,
                        rd,
                        rn,
                        set_cond,
                        operand2,
                    }),
                },
                None,
            )
        },
    )(input)
}

fn parse_multiply(input: &str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    map(
        tuple((
            terminated(alt((tag("mul"), tag("mla"))), char(' ')),
            terminated(parse_reg, char(',')),
            terminated(parse_reg, char(',')),
            terminated(parse_reg, char(',')),
            opt(terminated(parse_reg, char(','))),
        )),
        |(opcode, rd, rm, rs, opt_rn)| {
            let (accumulate, rn) = match (opcode, opt_rn) {
                ("mla", Some(rn)) => (true, rn),
                ("mul", None) => (false, 0),
                _ => unreachable!(),
            };

            (
                ConditionalInstruction {
                    cond: ConditionCode::Al,
                    instruction: Instruction::Multiply(InstructionMultiply {
                        rd,
                        rm,
                        rs,
                        rn,
                        accumulate,
                        set_cond: false,
                    }),
                },
                None,
            )
        },
    )(input)
}

fn parse_transfer(
    current_address: u32,
    next_free_address: u32,
) -> impl Fn(&str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    move |input: &str| {
        alt((
            parse_transfer_immediate(current_address, next_free_address),
            map(
                tuple((
                    alt((value(true, tag("ldr")), value(false, tag("str")))),
                    parse_reg,
                    alt((
                        delimited(
                            char('['),
                            tuple((parse_reg, parse_addressing_spec, success(true))),
                            char(']'),
                        ),
                        tuple((
                            delimited(char('['), parse_reg, char(']')),
                            parse_addressing_spec,
                            success(false),
                        )),
                    )),
                )),
                |(load, rd, (rn, (up_bit, offset), is_preindexed))| {
                    (
                        ConditionalInstruction {
                            cond: ConditionCode::Al,
                            instruction: Instruction::Transfer(InstructionTransfer {
                                is_preindexed,
                                up_bit,
                                load,
                                rd,
                                rn,
                                offset,
                            }),
                        },
                        None,
                    )
                },
            ),
        ))(input)
    }
}

fn parse_transfer_immediate(
    current_address: u32,
    next_free_address: u32,
) -> impl Fn(&str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    move |input: &str| {
        map(
            tuple((
                terminated(tag("ldr"), char(' ')),
                terminated(parse_reg, char(',')),
                preceded(char('='), alt((hexedecimal_value, decimal_value))),
            )),
            |(_, rd, expression)| {
                if expression <= 0xff {
                    (
                        ConditionalInstruction {
                            cond: ConditionCode::Al,
                            instruction: Instruction::Processing(InstructionProcessing {
                                opcode: ProcessingOpcode::Mov,
                                set_cond: false,
                                rd,
                                rn: 0,
                                operand2: expression_to_operand2(expression).unwrap(),
                            }),
                        },
                        None,
                    )
                } else {
                    let offset: i32 = next_free_address as i32 - current_address as i32 + 8;
                    (
                        ConditionalInstruction {
                            cond: ConditionCode::Al,
                            instruction: Instruction::Transfer(InstructionTransfer {
                                is_preindexed: true,
                                up_bit: true,
                                load: true,
                                rn: 15,
                                rd,
                                offset: expression_to_operand2(offset).unwrap(),
                            }),
                        },
                        Some(expression as u32),
                    )
                }
            },
        )(input)
    }
}

fn parse_addressing_spec(input: &str) -> NomResult<&str, (bool, Operand2)> {
    map(
        opt(tuple((
            char(','),
            opt(alt((tag("+"), tag("-")))),
            parse_operand2,
        ))),
        |opt_offset| match opt_offset {
            Some((_, Some(sign), op2)) => (sign == "+", op2),
            Some((_, None, op2)) => (true, op2),
            None => (true, Operand2::ConstantShift(0, 0)),
        },
    )(input)
}

fn parse_branch(
    current_address: u32,
    symbol_table: Rc<HashMap<String, u32>>,
) -> impl Fn(&str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    let st = symbol_table.clone();
    move |input: &str| {
        map(
            tuple((
                delimited(char('b'), opt(parse_condition_code), char(' ')),
                alt((
                    map(decimal_value, |x: i32| x.try_into().unwrap()),
                    map(alpha1, |label: &str| *st.get(label).unwrap()),
                )),
            )),
            |(opt_cond, addr)| {
                let cond = opt_cond.unwrap_or(ConditionCode::Al);
                let offset: i32 = ((addr as i32) - (current_address as i32) - 8) >> 2;

                (
                    ConditionalInstruction {
                        cond,
                        instruction: Instruction::Branch(InstructionBranch { offset }),
                    },
                    None,
                )
            },
        )(input)
    }
}

fn parse_operand2(input: &str) -> NomResult<&str, Operand2> {
    alt((parse_operand2_constant, parse_operand2_shifted))(input)
}

fn parse_operand2_constant(input: &str) -> NomResult<&str, Operand2> {
    map_opt(parse_expression, |value| expression_to_operand2(value).ok())(input)
}

fn expression_to_operand2(value: i32) -> Result<Operand2> {
    let mut rotate_count: u8 = 0x10;

    // If the value fits in 8 bits, we don't need to rotate it
    if value > 0xff {
        // While the least significant bits are both zeroes,
        // shift right and count a rotation.
        while value & 0x3 == 0 {
            value = value.overflowing_shr(2).0;
            rotate_count -= 1;
        }
    }

    // If the rotate count was not decremented, we take 0
    rotate_count &= 0xF;
    let to_rotate = value.try_into()?;
    Ok(Operand2::ConstantShift(rotate_count, to_rotate))
}

fn parse_operand2_shifted(input: &str) -> NomResult<&str, Operand2> {
    map(
        tuple((parse_reg, opt(preceded(char(','), parse_shift)))),
        |(reg_to_shift, shift_opt)| {
            shift_opt.map_or(
                Operand2::ShiftedReg(reg_to_shift, Shift::ConstantShift(ShiftType::Ror, 0)),
                |shift| Operand2::ShiftedReg(reg_to_shift, shift),
            )
        },
    )(input)
}

fn parse_shift(input: &str) -> NomResult<&str, Shift> {
    let (rest, shift_type) = parse_shifttype(input)?;
    preceded(
        char(' '),
        alt((
            map(parse_reg, move |reg: u8| {
                Shift::RegisterShift(shift_type, reg)
            }),
            map(parse_expression, move |x: i32| {
                Shift::ConstantShift(shift_type, x.try_into().unwrap())
            }),
        )),
    )(rest)
}

fn parse_reg(input: &str) -> NomResult<&str, u8> {
    verify(
        map(preceded(char('r'), digit1), |r: &str| {
            r.parse::<u8>().unwrap()
        }),
        |&r| (0..NUM_REGS).contains(&(r as usize)),
    )(input)
}

fn parse_expression(input: &str) -> NomResult<&str, i32> {
    preceded(char('#'), alt((hexedecimal_value, decimal_value)))(input)
}

fn hexedecimal_value(input: &str) -> NomResult<&str, i32> {
    map_opt(
        preceded(tag("0x"), recognize(tuple((opt(char('-')), hex_digit1)))),
        |out: &str| i32::from_str_radix(out, 16).ok(),
    )(input)
}

fn decimal_value(input: &str) -> NomResult<&str, i32> {
    map_opt(recognize(tuple((opt(char('-')), digit1))), |out: &str| {
        i32::from_str_radix(out, 10).ok()
    })(input)
}

fn parse_shifttype(input: &str) -> NomResult<&str, ShiftType> {
    alt((
        value(ShiftType::Lsl, tag("lsl")),
        value(ShiftType::Lsr, tag("lsr")),
        value(ShiftType::Asr, tag("asr")),
        value(ShiftType::Ror, tag("ror")),
    ))(input)
}

fn parse_processing_opcode(input: &str) -> NomResult<&str, ProcessingOpcode> {
    alt((
        value(ProcessingOpcode::And, tag("and")),
        value(ProcessingOpcode::Eor, tag("eor")),
        value(ProcessingOpcode::Sub, tag("sub")),
        value(ProcessingOpcode::Rsb, tag("rsb")),
        value(ProcessingOpcode::Add, tag("add")),
        value(ProcessingOpcode::Tst, tag("tst")),
        value(ProcessingOpcode::Teq, tag("teq")),
        value(ProcessingOpcode::Cmp, tag("cmp")),
        value(ProcessingOpcode::Orr, tag("orr")),
        value(ProcessingOpcode::Mov, tag("mov")),
    ))(input)
}

fn parse_condition_code(input: &str) -> NomResult<&str, ConditionCode> {
    alt((
        value(ConditionCode::Eq, tag("eq")),
        value(ConditionCode::Ne, tag("ne")),
        value(ConditionCode::Ge, tag("ge")),
        value(ConditionCode::Lt, tag("lt")),
        value(ConditionCode::Gt, tag("gt")),
        value(ConditionCode::Le, tag("le")),
    ))(input)
}

/// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reg() {
        assert_eq!(parse_reg("r12").expect("parse reg failed").1, 12);
        assert!(parse_reg("r123").is_err())
    }

    #[test]
    fn test_parse_shifttype() {
        assert_eq!(
            parse_shifttype("lsl").expect("parse shifttype failed").1,
            ShiftType::Lsl
        );
        assert_eq!(
            parse_shifttype("lsr2341234")
                .expect("parse shifttype failed")
                .1,
            ShiftType::Lsr
        );
    }

    #[test]
    fn test_parse_processing_opcode() {
        assert_eq!(
            parse_processing_opcode("eor")
                .expect("parse shifttype failed")
                .1,
            ProcessingOpcode::Eor
        );
        assert_eq!(
            parse_processing_opcode("mov2341234")
                .expect("parse shifttype failed")
                .1,
            ProcessingOpcode::Mov
        );
    }

    #[test]
    fn test_parse_expression() {
        assert_eq!(
            parse_expression("#123456")
                .expect("parse expression failed")
                .1,
            123456
        );
        assert_eq!(
            parse_expression("#-123456")
                .expect("parse expression failed")
                .1,
            -123456
        );
        assert_eq!(
            parse_expression("#0x123456")
                .expect("parse expression failed")
                .1,
            0x123456
        );
        assert_eq!(
            parse_expression("#0x-123456")
                .expect("parse expression failed")
                .1,
            -0x123456
        );
    }

    #[test]
    fn test_parse_operand2_constant() {
        // Check the case where the constant is less than 0xff
        assert_eq!(
            parse_operand2_constant("#0x2")
                .expect("parse operand 2 constant failed")
                .1,
            Operand2::ConstantShift(0, 0x2)
        );

        assert_eq!(
            parse_operand2_constant("#0x3f00000")
                .expect("parse operand 2 constant failed")
                .1,
            Operand2::ConstantShift(6, 0x3f)
        );
    }

    #[test]
    fn test_parse_operand2_shifted() {
        assert_eq!(
            parse_operand2_shifted("r2,lsr #2")
                .expect("parse operand 2 shifted failed")
                .1,
            Operand2::ShiftedReg(2, Shift::ConstantShift(ShiftType::Lsr, 2))
        )
    }

    #[test]
    fn test_parse_processing() {
        assert_eq!(
            parse_processing("add r3,r1,r2")
                .expect("parse processing failed")
                .1
                 .0,
            ConditionalInstruction {
                cond: ConditionCode::Al,
                instruction: Instruction::Processing(InstructionProcessing {
                    opcode: ProcessingOpcode::Add,
                    rd: 3,
                    rn: 1,
                    set_cond: false,
                    operand2: Operand2::ShiftedReg(2, Shift::ConstantShift(ShiftType::Ror, 0))
                })
            }
        );
    }
}
