use super::utils;
use crate::parse::*;
use crate::types::*;
use enum_primitive::FromPrimitive;
use nom::bits::bits as bits_fn;
use nom::{alt, bits, do_parse, named, tag_bits, take_bits};

pub fn decode(instr: &u32) -> Result<Instruction> {
    let mut bytes: &mut [u8];
    utils::to_u8_slice(*instr, bytes);
    Ok(parse_instr(bytes)?.1)
}

named!(
    parse_instr<Instruction>,
    alt!(parse_dataproc | parse_multiply | parse_sdt | parse_branch)
);

named!(
    parse_dataproc<Instruction>,
    bits!(do_parse!(
        op2: take_bits!(12u8)
            >> rd: take_bits!(4u8)
            >> rn: take_bits!(4u8)
            >> set_cond: take_bits!(1usize)
            >> opcode: take_bits!(4u8)
            >> is_immediate: take_bits!(1usize)
            >> tag_bits!(2u8, 0)
            >> cond: take_bits!(4u8)
            >> (Instruction::DataProc {
                cond: ConditionCode::from_u8(cond)
                    .ok_or(ArmNomError::new(ArmNomErrorKind::CondError))?,
                set_cond: set_cond == 1,
                opcode: DataProcOpcode::from_u8(opcode)
                    .ok_or(ArmNomError::new(ArmNomErrorKind::OpcodeError))?,
                rn,
                rd,
                operand2: if is_immediate == 1 {
                    bits_fn(parse_immediate_op2)(op2)?.1
                } else {
                    bits!(alt!(parse_constantreg_op2 | parse_shiftedreg_op2))?.1
                }
            })
    ))
);

named!(
    parse_multiply<Instruction>,
    bits!(do_parse!(
        rm: take_bits!(4u8)
            >> tag_bits!(4u8, 0x9)
            >> rs: take_bits!(4u8)
            >> rn: take_bits!(4u8)
            >> rd: take_bits!(4u8)
            >> set_cond: take_bits!(1usize)
            >> accumulate: take_bits!(1usize)
            >> tag_bits!(6u8, 0)
            >> cond: take_bits!(4u8)
            >> Instruction::Multiply {
                cond: ConditionCode::from_u8(cond)
                    .ok_or(format!("Can't create ConditionCode from {}", cond))?,
                accumulate: accumulate == 1,
                set_cond: set_cond == 1,
                rd,
                rn,
                rs,
                rm
            }
    ))
);

named!(
    parse_sdt<Instruction>,
    bits!(do_parse!(
        offset: take_bits!(12u8)
            >> rd: take_bits!(4u8)
            >> rn: take_bits!(4u8)
            >> load: take_bits!(1usize)
            >> tag_bits!(2usize, 0)
            >> up_bit: take_bits!(1usize)
            >> is_preindexed: take_bits!(1usize)
            >> is_shifted_r: take_bits!(1usize)
            >> tag_bits!(2usize, 0x1)
            >> cond: take_bits!(4u8)
            >> Instruction::SDT {
                cond: ConditionCode::from_u8(cond)
                    .ok_or(format!("Can't create ConditionCode from {}", cond))?,
                is_preindexed: is_preindexed == 1,
                up_bit: up_bit == 1,
                load: load == 1,
                rn,
                rd,
                offset: if is_shifted_r != 1 {
                    bits!(parse_immediate_op2!(op2))?.1
                } else {
                    bits!(alt!(parse_constantreg_op2 | parse_shiftedreg_op2))?.1
                }
            }
    ))
);

named!(
    parse_branch<Instruction>,
    bits!(do_parse!(
        offset: take_bits!(24usize)
            >> tag_bits!(4u8, 0xa)
            >> cond: take_bits!(4u8)
            >> Instruction::Branch {
                cond: ConditionCode::from_u8(cond)
                    .ok_or(format!("Can't create ConditionCode from {}", cond))?,
                offset as i32
            }
    ))
);

named!(
    parse_immediate_op2<Operand2>,
    bits!(do_parse!(
        to_shift: take_bits!(8u8)
            >> shift_amt: take_bits!(4u8)
            >> Operand2::ConstantShift(shift_amt, to_shift)
    ))
);

named!(
    parse_constantreg_op2<Operand2>,
    bits!(do_parse!(
        reg_to_shift: take_bits!(4u8)
            >> tag_bits!(1usize, 0)
            >> shift_type: take_bits!(2u8)
            >> constant_shift: take_bits!(5u8)
            >> Operand2::ConstantShiftedReg(
                constant_shift,
                ShiftType::from_u8(shift_type)
                    .ok_or(format!("Can't create shift_type from {}", shift_type))?,
                reg_to_shift
            )
    ))
);

named!(
    parse_shiftedreg_op2<Operand2>,
    bits!(do_parse!(
        reg_to_shift: take_bits!(4u8)
            >> tag_bits!(1usize, 1)
            >> shift_type: take_bits!(2u8)
            >> tag_bits!(1usize, 0)
            >> shift_reg: take_bits!(4u8)
            >> Operand2::ShiftedReg(
                shift_reg,
                ShiftType::from_u8(shift_type).ok_or(
                    format!("Can't create shift_type from {}", shift_type)?,
                    reg_to_shift
                )
            )
    ))
);
