use std::convert::TryInto;

use crate::{
    constants::*,
    types::{Instruction::*, *},
};

use super::state::*;
use super::utils;

pub fn execute(state: &mut EmulatorState, instr: ConditionalInstruction) -> Result<()> {
    if !instr.satisfies_cpsr(state.reg(EmulatorState::CPSR)) {
        return Ok(());
    }

    match instr.instruction {
        Processing(processing) => execute_processing(state, processing),
        Multiply(multiply) => execute_multiply(state, multiply),
        Transfer(transfer) => execute_transfer(state, transfer),
        Branch(branch) => execute_branch(state, branch),
    }
}

fn execute_processing(state: &mut EmulatorState, instr: InstructionProcessing) -> Result<()> {
    let InstructionProcessing {
        cond,
        opcode,
        set_cond,
        rn,
        rd,
        operand2,
    } = instr;

    // Get operands
    let op1 = state.reg(rn as usize);
    let (op2, bs_carry_out) = barrel_shifter(operand2, state.regs_mut());

    // Perform process
    let (result, carry_out) =
        perform_processing_operation((*op1).try_into()?, op2.try_into()?, opcode);

    // Save result
    match opcode {
        ProcessingOpcode::Cmp | ProcessingOpcode::Teq | ProcessingOpcode::Tst => (),
        _ => {
            state.regs_mut()[rd as usize] = result as u32;
        }
    }

    // Set flags
    if set_cond {
        let c_flag = !bs_carry_out & carry_out | carry_out;
        state.set_flags(CpsrFlag::CFlag, c_flag);
        state.set_flags(
            CpsrFlag::NFlag,
            utils::extract_bit(&(result as u32), CpsrFlag::NFlag as u8),
        );
        state.set_flags(CpsrFlag::ZFlag, result == 0);
    }

    Ok(())
}

fn execute_multiply(state: &mut EmulatorState, instr: InstructionMultiply) -> Result<()> {
    let InstructionMultiply {
        accumulate,
        set_cond,
        rd,
        rn,
        rs,
        rm,
    } = instr;

    // Perform multiplication
    let mut result: u32 = state.reg(rm as usize) * state.reg(rs as usize);

    if accumulate {
        result += state.reg(rn as usize);
    }

    // Save result
    state.regs_mut()[rd as usize] = result;

    // Set flags
    if set_cond {
        state.set_flags(
            CpsrFlag::NFlag,
            utils::extract_bit(&result, CpsrFlag::NFlag as u8),
        );
        state.set_flags(CpsrFlag::ZFlag, result == 0);
    }

    Ok(())
}

fn execute_transfer(state: &mut EmulatorState, instr: InstructionTransfer) -> Result<()> {
    let InstructionTransfer {
        is_preindexed,
        up_bit,
        load,
        rn,
        rd,
        offset,
    } = instr;

    // Calculate offset
    let interpreted_offset: i32 = match offset {
        Operand2::ConstantShift(rotate, imm) => i32::from(rotate) << 8 | i32::from(imm),
        _ => barrel_shifter(offset, state.regs_mut()).0 as i32,
    };

    // Calculate memory address
    let mut mem_address: usize = (*state.reg(rn as usize)).try_into()?;

    // Handle pre-indexing
    if is_preindexed {
        mem_address += if up_bit {
            interpreted_offset
        } else {
            -1 * interpreted_offset
        } as usize;
    }

    // Perform transfer
    if mem_address <= MEMORY_SIZE {
        if load {
            // Load the memory to R[rd]
            state.regs_mut()[rd as usize] = *state.read_memory(mem_address);
        } else {
            // Stores the value at Mem[rd]
            state.write_memory(mem_address, state.regs_mut()[rd as usize])
        }
    } else {
        println!(
            "Error: Out of bounds memory access at 0x{:0>8x}",
            mem_address
        );
    }

    // Handle post-indexing
    if !is_preindexed {
        state.regs_mut()[rn as usize] += if up_bit {
            interpreted_offset
        } else {
            -1 * interpreted_offset
        } as u32;
    }

    Ok(())
}

fn execute_branch(state: &mut EmulatorState, instr: InstructionBranch) -> Result<()> {
    let InstructionBranch { cond, offset } = instr;

    // Update the PC
    let PC = &mut state.regs_mut()[EmulatorState::PC];
    *PC = (*PC as i32 + utils::signed_24_to_32(offset)) as u32;

    // Flush the pipeline
    state.pipeline_mut().flush();

    Ok(())
}

/// Helper Functions and Impls

impl ConditionalInstruction {
    fn satisfies_cpsr(&self, cpsr_contents: &u32) -> bool {
        let n: bool = utils::extract_bit(cpsr_contents, 31);
        let z: bool = utils::extract_bit(cpsr_contents, 30);
        let v: bool = utils::extract_bit(cpsr_contents, 28);

        match self.cond {
            Eq => z,
            Ne => !z,
            Ge => n == v,
            Lt => n != v,
            Gt => !z && (n == v),
            Le => z || (n != v),
            Al => true,
        }
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
            (register_file[shift_reg as usize] & utils::mask(8)) as u8,
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
            utils::extract_bit(&to_shift, shift_amt - 1),
        ),
    }
}

pub fn perform_processing_operation(op1: i32, op2: i32, opcode: ProcessingOpcode) -> (i32, bool) {
    match opcode {
        ProcessingOpcode::And | ProcessingOpcode::Tst => (op1 & op2, false),
        ProcessingOpcode::Eor | ProcessingOpcode::Teq => (op1 ^ op2, false),
        ProcessingOpcode::Sub => op1.overflowing_sub(op2),
        ProcessingOpcode::Rsb => op2.overflowing_sub(op1),
        ProcessingOpcode::Add => op1.overflowing_add(op1),
        ProcessingOpcode::Cmp => (op1 - op2, !(op1 < op2)),
        ProcessingOpcode::Orr => (op1 | op2, false),
        ProcessingOpcode::Mov => (op2, false),
    }
}
