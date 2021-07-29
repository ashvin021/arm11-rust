use std::{collections::HashMap, convert::TryInto, rc::Rc};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, digit1, hex_digit1, space0, space1},
    combinator::{complete, map, map_opt, opt, recognize, success, value, verify},
    error::context,
    sequence::{delimited, preceded, terminated, tuple},
};

use crate::{constants::*, parse::*, types::*};

// Parses an ARM assembly instruction in the form of a string into a ConditionalInstruction. There
// are 4 main types of instructions:
// 1. Processing
// 2. Multiply
// 3. Transfer
// 4. Branch
//
// There are also 2 special cases; Halt and Lsl instructions.
//
// The second field in the return tuple may contain data (usually from Transfer instructions),
// which are to be added to the assembled binary, at the end of all the encoded instructions.
//
pub fn parse_asm(
    raw: &str,
    current_address: usize,
    next_free_address: usize,
    symbol_table: Rc<HashMap<String, u32>>,
) -> Result<(ConditionalInstruction, Option<u32>)> {
    let (instr, opt_data) = alt((
        complete(parse_halt),
        complete(parse_lsl),
        complete(parse_processing),
        complete(parse_transfer(current_address, next_free_address)),
        complete(parse_multiply),
        complete(parse_branch(current_address, symbol_table)),
    ))(raw)
    .map_err(|e| format!("{:#?}", e))?
    .1;

    Ok((instr, opt_data))
}

// Parses a processing instruction. This can either be:
//
// 1. Instructions that compute results: and, eor, sub, rsb, add, orr
// eg: <opcode> Rd,Rn,<Operand2>
//
// 2. Single operand assignment: mov
// eg: mov Rd,<Operand2>
//
// 3. Instructions that do not compute results, but do set CPSR flags: tst, teq, cmp
// eg: <opcode> Rn,<Operand2>
//
// This returns no additional data, so the second field of the return tuple will
// always be None.
//
fn parse_processing(input: &str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    let (rest, opcode) = context(
        "parsing processing opcode",
        terminated(parse_processing_opcode, space1),
    )(input)?;
    context(
        "parsing processing instruction",
        map(
            alt((
                tuple((
                    // case with two registers
                    // eg: <opcode> Rd,Rn,<Operand2>
                    terminated(parse_reg, comma_space),
                    terminated(parse_reg, comma_space),
                    parse_operand2,
                    success(false),
                )),
                tuple((
                    // cases with one register
                    // eg: mov Rd,<Operand2>
                    // eg: <opcode> Rn,<Operand2>
                    success(0),
                    terminated(parse_reg, comma_space),
                    parse_operand2,
                    success(true),
                )),
            )),
            move |(r1, r2, (operand2, _), set_cond)| {
                // If its a Mov instruction, the result is saved to Rd, instead of Rn
                let (rd, rn, set_cond) = match opcode {
                    ProcessingOpcode::Mov => (r2, r1, false),
                    _ => (r1, r2, set_cond),
                };
                (
                    ConditionalInstruction {
                        cond: ConditionCode::Al,
                        instruction: Instruction::Processing(InstructionProcessing {
                            opcode,
                            set_cond,
                            rn,
                            rd,
                            operand2,
                        }),
                    },
                    None,
                )
            },
        ),
    )(rest)
}

// Parses a multiply instruction. This can either be a multiply instruction (mul Rd,Rm,Rs)
// or an multiply with accumulate instruction (mla Rd,Rm,Rs,Rn).
//
// This returns no additional data, so the second field of the return tuple will
// always be None.
//
fn parse_multiply(input: &str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    context(
        "parsing multiply instruction",
        map(
            tuple((
                terminated(alt((tag("mul"), tag("mla"))), space1),
                terminated(parse_reg, comma_space),
                terminated(parse_reg, comma_space),
                parse_reg,
                opt(preceded(comma_space, parse_reg)),
            )),
            |(opcode, rd, rm, rs, opt_rn)| {
                // Mla instructions are accumulate, and have an Rn register specified
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
        ),
    )(input)
}

// Parses a transfer instruction. This can either be an immediate expression, or an indexed
// instruction.
//
// This may return additional data in the Option<u32>.
//
fn parse_transfer(
    current_address: usize,
    next_free_address: usize,
) -> impl Fn(&str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    move |input: &str| {
        context(
            "parsing transfer instruction",
            alt((
                parse_transfer_immediate(current_address, next_free_address),
                parse_transfer_indexed,
            )),
        )(input)
    }
}

// Returns a parser for an immediate transfer instruction, given the address of the current
// instruction, and the next address available for data.
//
// If the immediate expression can fit inside of a mov instruction, this is interpreted as
// so, and the parser returns a mov instruction with no additional data.
// If the expression cannot fit inside a mov instruction, it is returned by the parser as
// additional data in the Option<u32>. The instruction is a transfer instruction which
// contains the offset to the address of this data.
//
fn parse_transfer_immediate(
    current_address: usize,
    next_free_address: usize,
) -> impl Fn(&str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    move |input: &str| {
        context(
            "parsing immediate transfer",
            map(
                tuple((
                    terminated(tag("ldr"), space1),
                    terminated(parse_reg, comma_space),
                    preceded(char('='), alt((hexedecimal_value, decimal_value))),
                )),
                |(_, rd, (expression, _))| {
                    if expression <= mask(IMM_VALUE.size as u8) {
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
                        let offset: i32 = next_free_address as i32
                            - (current_address as i32 + PIPELINE_OFFSET as i32);
                        (
                            ConditionalInstruction {
                                cond: ConditionCode::Al,
                                instruction: Instruction::Transfer(InstructionTransfer {
                                    is_preindexed: true,
                                    up_bit: true,
                                    load: true,
                                    rn: PC as u8,
                                    rd,
                                    offset: expression_to_operand2(offset as u32).unwrap(),
                                }),
                            },
                            Some(expression as u32),
                        )
                    }
                },
            ),
        )(input)
    }
}

// Parses an indexed transfer instruction. This can be without an offset (eg: <opcode> [Rd]), with
// a pre-indexed offset (eg: <opcode> [Rd, <Operand2>]) or with a post-indexed offset (eg: <opcode>
// [Rd] <Operand2>).
//
// This returns no additional data, so the second field of the return tuple will
// always be None.
//
fn parse_transfer_indexed(input: &str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    context(
        "parsing indexed transfer",
        map(
            tuple((
                terminated(
                    alt((value(true, tag("ldr")), value(false, tag("str")))),
                    space1,
                ),
                terminated(parse_reg, comma_space),
                alt((
                    // Post-indexed case
                    // eg: <opcode> [Rd], <Operand2>
                    context(
                        "parsing post-indexed transfer, with offset",
                        complete(tuple((
                            delimited(char('['), parse_reg, char(']')),
                            preceded(comma_space, parse_operand2),
                            success(false),
                        ))),
                    ),
                    // Pre-indexed case
                    // eg: <opcode> [Rd, <Operand2>]
                    context(
                        "parsing pre-indexed transfer, with offset",
                        complete(delimited(
                            char('['),
                            tuple((
                                parse_reg,
                                preceded(comma_space, parse_operand2),
                                success(true),
                            )),
                            char(']'),
                        )),
                    ),
                    // Default case, pre-indexed with no addressing offset
                    // eg: <opcode> [Rd]
                    context(
                        "parsing pre-indexed transfer, with no offset",
                        complete(tuple((
                            delimited(char('['), parse_reg, char(']')),
                            success((Operand2::ConstantShift(0, 0), false)),
                            success(true),
                        ))),
                    ),
                )),
            )),
            |(load, rd, (rn, (offset, is_signed), is_preindexed))| {
                (
                    ConditionalInstruction {
                        cond: ConditionCode::Al,
                        instruction: Instruction::Transfer(InstructionTransfer {
                            is_preindexed,
                            up_bit: !is_signed,
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
    )(input)
}

// Returns a parser for branch instructions, given the address of the current instruction and the
// symbol table.
//
// The parser will return no additional data, so the second field of the parser's return tuple will
// always be None.
//
fn parse_branch(
    current_address: usize,
    symbol_table: Rc<HashMap<String, u32>>,
) -> impl Fn(&str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    move |input: &str| {
        context(
            "parsing branch instruction",
            map(
                tuple((
                    delimited(char('b'), opt(parse_condition_code), space1),
                    alt((
                        // Direct branch address, given as a decimal integer
                        context(
                            "parsing direct branch offset",
                            map_opt(signed_decimal_value, |x: i32| x.try_into().ok()),
                        ),
                        // Label branch address, lookup in symbol table
                        context(
                            "parsing label branch offset",
                            map_opt(alphanumeric1, |label: &str| {
                                symbol_table.get(label).copied()
                            }),
                        ),
                    )),
                )),
                |(opt_cond, addr)| {
                    let cond = opt_cond.unwrap_or(ConditionCode::Al);
                    let offset: i32 =
                        (addr as i32 - current_address as i32 - PIPELINE_OFFSET as i32) >> 2;

                    (
                        ConditionalInstruction {
                            cond,
                            instruction: Instruction::Branch(InstructionBranch { offset }),
                        },
                        None,
                    )
                },
            ),
        )(input)
    }
}

// Parses a halt instruction, i.e. andeq r0,r0,r0.
//
// This returns no additional data, so the second field of the return tuple will
// always be None.
//
fn parse_halt(input: &str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    context(
        "parsing halt instruction",
        value(
            (
                ConditionalInstruction {
                    cond: ConditionCode::Eq,
                    instruction: Instruction::Halt,
                },
                None,
            ),
            recognize(tuple((
                tag("andeq"),
                space1,
                tag("r0"),
                comma_space,
                tag("r0"),
                comma_space,
                tag("r0"),
            ))),
        ),
    )(input)
}

// Parses an lsl instruction. This provides an ARM assembly compatible way of shifting registers,
// without supporting the full syntax for shift-modified expressions.
//
// This returns no additional data, so the second field of the return tuple will
// always be None.
//
fn parse_lsl(input: &str) -> NomResult<&str, (ConditionalInstruction, Option<u32>)> {
    let (rest, (rn, op2)) = context(
        "parsing lsl instruction operands",
        tuple((
            delimited(tag("lsl "), parse_reg, char(',')),
            recognize(parse_operand2_constant),
        )),
    )(input)?;

    // The lsl instruction is desugared into a mov instruction, which is then parsed.
    let desugared = format!("mov r{},r{}, lsl {}", rn, rn, op2);
    let parsed = context("parsing lsl instruction as mov", parse_processing)(desugared.as_str())
        .expect("parse failed")
        .1;

    Ok((rest, parsed))
}

// Parses an Operand2 from a string. This can be either a constant shifted or a register shifted value.
fn parse_operand2(input: &str) -> NomResult<&str, (Operand2, bool)> {
    context(
        "parsing operand2",
        alt((parse_operand2_constant, parse_operand2_shifted)),
    )(input)
}

// Parses an expression from a string, directly to an Operand2.
fn parse_operand2_constant(input: &str) -> NomResult<&str, (Operand2, bool)> {
    let (rest, (value, is_signed)) = context("parsing operand2 constant", parse_expression)(input)?;
    let op2 = expression_to_operand2(value)
        .map_err(|_| ArmNomError::new(ArmNomErrorKind::Operand2Constant))?;

    Ok((rest, (op2, is_signed)))
}

// Converts u32 to a constant shifted Operand2.
//
// assert_eq!(expression_to_operand2(0x2), Operand2::ConstantShift(0x2, 0));
// assert_eq!(expression_to_operand2(0x3f0000), Operand2::ConstantShift(0x3f, 6));
//
fn expression_to_operand2(mut value: u32) -> Result<Operand2> {
    let mut rotate_count: u8 = 1 << 4;

    // If the value fits in 8 bits, we don't need to rotate it
    if value > mask(IMM_VALUE.size as u8) {
        // While the least significant bits are both zeroes,
        // shift right and count a rotation.
        while value & mask(2) == 0 {
            value = value.overflowing_shr(2).0;
            rotate_count -= 1;
        }
    }

    // If the rotate count was not decremented, we take 0
    rotate_count &= mask(4) as u8;
    let to_rotate = value.try_into()?;
    Ok(Operand2::ConstantShift(to_rotate, rotate_count))
}

// Parses a shifted register Operand2, i.e a string of the form: <register>{, <shift>}
// Curly braces here indicate that the shift is optional. If no shift is given, we use a constant
// shift of 0 as the shift value.
//
fn parse_operand2_shifted(input: &str) -> NomResult<&str, (Operand2, bool)> {
    context(
        "parsing operand2 shifted",
        map(
            tuple((parse_reg, opt(preceded(comma_space, parse_shift)))),
            |(reg_to_shift, shift_opt)| {
                (
                    shift_opt.map_or(
                        Operand2::ShiftedReg(reg_to_shift, Shift::ConstantShift(ShiftType::Lsl, 0)),
                        |shift| Operand2::ShiftedReg(reg_to_shift, shift),
                    ),
                    false,
                )
            },
        ),
    )(input)
}

// Parses a shift, i.e. an expression which is either a <shifttype> <#expression> or a
// <shifttype> <register>. It is preceded by 0 or more spaces.
//
// assert_eq!(parse_shift("  lsl r2"), Ok("", Shift::RegisterShift(ShiftType::Lsl, 2)));
// assert_eq!(parse_shift("ror #2")), Ok("", Shift::ConstantShift(ShiftType::Ror, 2));
//
fn parse_shift(input: &str) -> NomResult<&str, Shift> {
    let (rest, shift_type) = context("parsing shift type", parse_shifttype)(input)?;
    context(
        "parsing shift",
        preceded(
            space0,
            alt((
                map(parse_expression, move |(x, _)| {
                    Shift::ConstantShift(shift_type, x.try_into().unwrap())
                }),
                map(parse_reg, move |reg: u8| {
                    Shift::RegisterShift(shift_type, reg)
                }),
            )),
        ),
    )(rest)
}

// Parses a register of the form r<int>, where int is a valid available register
// eg: r0, r12, 15
//
fn parse_reg(input: &str) -> NomResult<&str, u8> {
    context(
        "parsing register",
        verify(
            map_opt(preceded(char('r'), digit1), |r: &str| r.parse::<u8>().ok()),
            |&r| {
                (0..NUM_GENERAL_REGS).contains(&(r as usize))
                    || r as usize == PC
                    || r as usize == CPSR
            },
        ),
    )(input)
}

fn parse_expression(input: &str) -> NomResult<&str, (u32, bool)> {
    context(
        "parsing expresssion",
        preceded(char('#'), alt((hexedecimal_value, decimal_value))),
    )(input)
}

// Parses a signed hexadecimal value to a (u32, bool), where the boolean is true if the
// original value is negative.
// eg:
//
// assert_eq!(hexedecimal_value("0x1234"), Ok("", (0x1234, false))
// assert_eq!(hexedecimal_value("-0x6969"), Ok("", (0x6969, true))
//
fn hexedecimal_value(input: &str) -> NomResult<&str, (u32, bool)> {
    let (rest, (opt_sign, out)) = context(
        "parsing hexedecimal value",
        tuple((opt(char('-')), preceded(tag("0x"), recognize(hex_digit1)))),
    )(input)?;

    Ok((
        rest,
        (
            u32::from_str_radix(out, 16)
                .map_err(|_| ArmNomError::new(ArmNomErrorKind::HexadecimalValue))?,
            opt_sign.is_some(),
        ),
    ))
}

// Parses a signed decimal value to a (u32, bool), where the boolean is true if the
// original value is negative.
//
// assert_eq!(hexedecimal_value("1234"), Ok("", (1234, false))
// assert_eq!(hexedecimal_value("-6969"), Ok("", (6969, true))
//
fn decimal_value(input: &str) -> NomResult<&str, (u32, bool)> {
    let (rest, (opt_sign, out)) = context(
        "parsing decimal value",
        tuple((opt(char('-')), recognize(digit1))),
    )(input)?;

    Ok((
        rest,
        (
            out.parse::<u32>()
                .map_err(|_| ArmNomError::new(ArmNomErrorKind::DecimalValue))?,
            opt_sign.is_some(),
        ),
    ))
}

// Parses a signed hexadecimal value to an i32.
fn signed_decimal_value(input: &str) -> NomResult<&str, i32> {
    let (rest, out) = context(
        "parsing signed decimal value",
        recognize(tuple((opt(char('-')), digit1))),
    )(input)?;

    Ok((
        rest,
        out.parse::<i32>()
            .map_err(|_| ArmNomError::new(ArmNomErrorKind::SignedDecimalValue))?,
    ))
}

// Matches a comma, with 0 or more spaces following it.
fn comma_space(input: &str) -> NomResult<&str, char> {
    terminated(char(','), space0)(input)
}

// Parses shifttype strings into values of ShiftType.
fn parse_shifttype(input: &str) -> NomResult<&str, ShiftType> {
    context(
        "parsing shift type",
        alt((
            value(ShiftType::Lsl, tag("lsl")),
            value(ShiftType::Lsr, tag("lsr")),
            value(ShiftType::Asr, tag("asr")),
            value(ShiftType::Ror, tag("ror")),
        )),
    )(input)
}

// Parses processing opcode strings into values of ProcessingOpcode.
fn parse_processing_opcode(input: &str) -> NomResult<&str, ProcessingOpcode> {
    context(
        "parsing processing opcode",
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
        )),
    )(input)
}

// Parses condition code strings into values of ConditionCode.
fn parse_condition_code(input: &str) -> NomResult<&str, ConditionCode> {
    context(
        "parsing condition code",
        alt((
            value(ConditionCode::Eq, tag("eq")),
            value(ConditionCode::Ne, tag("ne")),
            value(ConditionCode::Ge, tag("ge")),
            value(ConditionCode::Lt, tag("lt")),
            value(ConditionCode::Gt, tag("gt")),
            value(ConditionCode::Le, tag("le")),
        )),
    )(input)
}

///////////////////////////////////////////////////////////////////////////////
// TESTS
///////////////////////////////////////////////////////////////////////////////

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
            (123456, false)
        );
        assert_eq!(
            parse_expression("#-123456")
                .expect("parse expression failed")
                .1,
            (123456, true)
        );
        assert_eq!(
            parse_expression("#0x123456")
                .expect("parse expression failed")
                .1,
            (0x123456, false)
        );
        assert_eq!(
            parse_expression("#-0x123456")
                .expect("parse expression failed")
                .1,
            (0x123456, true)
        );
    }

    #[test]
    fn test_parse_operand2_constant() {
        // Check the case where the constant is less than IMM_VALUE.size
        assert_eq!(
            parse_operand2_constant("#0x2")
                .expect("parse operand 2 constant failed")
                .1,
            (Operand2::ConstantShift(0x2, 0), false)
        );

        assert_eq!(
            parse_operand2_constant("#0x3f00000")
                .expect("parse operand 2 constant failed")
                .1,
            (Operand2::ConstantShift(0x3f, 6), false)
        );
    }

    #[test]
    fn test_parse_operand2_shifted() {
        assert_eq!(
            parse_operand2_shifted("r2,lsr #2")
                .expect("parse operand 2 shifted failed")
                .1,
            (
                Operand2::ShiftedReg(2, Shift::ConstantShift(ShiftType::Lsr, 2)),
                false
            )
        )
    }

    #[test]
    fn test_parse_processing() {
        assert_eq!(
            parse_processing("add r3,r1,r2")
                .expect("parse processing failed")
                .1,
            (
                ConditionalInstruction {
                    cond: ConditionCode::Al,
                    instruction: Instruction::Processing(InstructionProcessing {
                        opcode: ProcessingOpcode::Add,
                        rd: 3,
                        rn: 1,
                        set_cond: false,
                        operand2: Operand2::ShiftedReg(2, Shift::ConstantShift(ShiftType::Lsl, 0))
                    })
                },
                None
            )
        );
    }

    #[test]
    fn test_parse_multiply() {
        assert_eq!(
            parse_multiply("mul r3,r1,r2")
                .expect("parse multiply failed")
                .1,
            (
                ConditionalInstruction {
                    cond: ConditionCode::Al,
                    instruction: Instruction::Multiply(InstructionMultiply {
                        accumulate: false,
                        set_cond: false,
                        rd: 3,
                        rm: 1,
                        rs: 2,
                        rn: 0
                    })
                },
                None
            )
        );

        assert_eq!(
            parse_multiply("mla r3,r1,r2,r4")
                .expect("parse multiply failed")
                .1,
            (
                ConditionalInstruction {
                    cond: ConditionCode::Al,
                    instruction: Instruction::Multiply(InstructionMultiply {
                        accumulate: true,
                        set_cond: false,
                        rd: 3,
                        rm: 1,
                        rs: 2,
                        rn: 4
                    })
                },
                None
            )
        );
    }

    #[test]
    fn test_parse_branch() {
        let mut symbol_table = HashMap::new();
        symbol_table.insert("foo".to_owned(), 0x14);
        symbol_table.insert("wait".to_owned(), 0x4);
        let rc_symbol_table = Rc::new(symbol_table);

        let st_1 = rc_symbol_table.clone();
        assert_eq!(
            parse_branch(0xc, st_1)("beq foo")
                .expect("parse branch failed")
                .1,
            (
                ConditionalInstruction {
                    cond: ConditionCode::Eq,
                    instruction: Instruction::Branch(InstructionBranch { offset: 0 })
                },
                None
            )
        );

        let st_2 = rc_symbol_table.clone();
        assert_eq!(
            parse_branch(0xc, st_2)("bne wait")
                .expect("parse branch failed")
                .1,
            (
                ConditionalInstruction {
                    cond: ConditionCode::Ne,
                    instruction: Instruction::Branch(InstructionBranch { offset: -4 })
                },
                None
            )
        );
    }

    #[test]
    fn test_parse_transfer_immediate() {
        // Case where expression <= IMM_VALUE.size
        assert_eq!(
            parse_transfer_immediate(0x0, 0xc)("ldr r0,=0x02")
                .expect("parse transfer failed")
                .1,
            (
                ConditionalInstruction {
                    cond: ConditionCode::Al,
                    instruction: Instruction::Processing(InstructionProcessing {
                        opcode: ProcessingOpcode::Mov,
                        set_cond: false,
                        rn: 0,
                        rd: 0,
                        operand2: Operand2::ConstantShift(0x02, 0)
                    })
                },
                None
            )
        );

        // Case where expression > IMM_VALUE.size
        assert_eq!(
            parse_transfer_immediate(0x0, 0x8)("ldr r2,=0x20200020")
                .expect("parse transfer immediate failed")
                .1,
            (
                ConditionalInstruction {
                    cond: ConditionCode::Al,
                    instruction: Instruction::Transfer(InstructionTransfer {
                        is_preindexed: true,
                        up_bit: true,
                        load: true,
                        rn: PC as u8,
                        rd: 2,
                        offset: Operand2::ConstantShift(0x0, 0),
                    })
                },
                Some(0x20200020)
            )
        )
    }

    #[test]
    fn test_parse_halt() {
        assert_eq!(
            parse_halt("andeq r0,r0,r0").expect("parse halt failed").1,
            (
                ConditionalInstruction {
                    cond: ConditionCode::Eq,
                    instruction: Instruction::Halt
                },
                None
            )
        );
    }
}
