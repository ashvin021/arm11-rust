use std::convert::TryInto;

use crate::{
    constants::*,
    types::{Instruction::*, *},
};

use super::{gpio::*, state::*};

pub fn execute(state: &mut EmulatorState, instr: ConditionalInstruction) -> Result<()> {
    if !instr.satisfies_cpsr(state.read_reg(CPSR)) {
        return Ok(());
    }

    match instr.instruction {
        Processing(processing) => execute_processing(state, processing),
        Multiply(multiply) => execute_multiply(state, multiply),
        Transfer(transfer) => execute_transfer(state, transfer),
        Branch(branch) => execute_branch(state, branch),
        Halt => panic!("Can't execute halt"),
    }
}

fn execute_processing(state: &mut EmulatorState, instr: InstructionProcessing) -> Result<()> {
    let InstructionProcessing {
        opcode,
        set_cond,
        rn,
        rd,
        operand2,
    } = instr;

    // Get operands
    let op1 = state.read_reg(rn as usize);
    let (op2, bs_carry_out) = barrel_shifter(operand2, state.regs());
    // Perform process
    let (result, carry_out) = perform_processing_operation((*op1) as i32, op2 as i32, opcode);

    // Save result
    match opcode {
        ProcessingOpcode::Cmp | ProcessingOpcode::Teq | ProcessingOpcode::Tst => (),
        _ => {
            state.write_reg(rd as usize, result as u32);
        }
    }

    // Set flags
    if set_cond {
        let c_flag = !bs_carry_out & carry_out | carry_out;
        state.set_flags(CpsrFlag::C, c_flag);
        state.set_flags(
            CpsrFlag::N,
            extract_bit(&(result as u32), CpsrFlag::N as u8),
        );
        state.set_flags(CpsrFlag::Z, result == 0);
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
    let mut result: u32 = state.read_reg(rm as usize) * state.read_reg(rs as usize);

    if accumulate {
        result += state.read_reg(rn as usize);
    }

    // Save result
    state.write_reg(rd as usize, result);

    // Set flags
    if set_cond {
        state.set_flags(CpsrFlag::N, extract_bit(&result, CpsrFlag::N as u8));
        state.set_flags(CpsrFlag::Z, result == 0);
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
        Operand2::ConstantShift(imm, rotate) => i32::from(rotate) << IMM_SHIFT.pos | i32::from(imm),
        _ => barrel_shifter(offset, state.regs()).0 as i32,
    };

    // Calculate memory address
    let mut mem_address: usize = (*state.read_reg(rn as usize)).try_into()?;

    // Handle pre-indexing
    if is_preindexed {
        mem_address = ((mem_address as i32)
            + if up_bit {
                interpreted_offset
            } else {
                -interpreted_offset
            }) as usize;
    }

    // Perform transfer
    const LAST_MEM: usize = MEMORY_SIZE - 1;
    match mem_address {
        0..=LAST_MEM => {
            if load {
                // Load the memory to R[rd]
                state.write_reg(rd as usize, state.read_memory(mem_address)?);
            } else {
                // Stores the value at Mem[rd]
                state.write_memory(mem_address, state.regs()[rd as usize])
            }
        }
        _ if gpio_accessed(mem_address) => {
            print_gpio_message(mem_address);
            if load {
                state.write_reg(rd as usize, mem_address as u32);
            }
        }
        _ => println!(
            "Error: Out of bounds memory access at address 0x{:0>8x}",
            mem_address
        ),
    }

    // Handle post-indexing
    if !is_preindexed {
        let mut rn_val = *state.read_reg(rn as usize);
        rn_val += if up_bit {
            interpreted_offset
        } else {
            -interpreted_offset
        } as u32;
        state.write_reg(rn as usize, rn_val);
    }

    Ok(())
}

fn execute_branch(state: &mut EmulatorState, instr: InstructionBranch) -> Result<()> {
    let InstructionBranch { offset } = instr;

    // Update the PC
    let mut pc = *state.read_reg(PC);
    pc = (pc as i32 + signed_24_to_32(offset << 2)) as u32;
    state.write_reg(PC, pc);

    // Flush the pipeline
    state.pipeline.flush();

    Ok(())
}

/// Helper Functions and Impls

impl ConditionalInstruction {
    fn satisfies_cpsr(&self, cpsr_contents: &u32) -> bool {
        let n: bool = extract_bit(cpsr_contents, CpsrFlag::N as u8);
        let z: bool = extract_bit(cpsr_contents, CpsrFlag::Z as u8);
        let v: bool = extract_bit(cpsr_contents, CpsrFlag::V as u8);

        match self.cond {
            ConditionCode::Eq => z,
            ConditionCode::Ne => !z,
            ConditionCode::Ge => n == v,
            ConditionCode::Lt => n != v,
            ConditionCode::Gt => !z && (n == v),
            ConditionCode::Le => z || (n != v),
            ConditionCode::Al => true,
        }
    }
}

pub fn barrel_shifter(op2: Operand2, register_file: &[u32; NUM_REGS]) -> (u32, bool) {
    let (to_shift, shift_amt, shift_type): (u32, u8, ShiftType) = match op2 {
        Operand2::ConstantShift(to_shift, shift_amt) => {
            (u32::from(to_shift), 2 * shift_amt, ShiftType::Ror)
        }
        Operand2::ShiftedReg(reg_to_shift, Shift::ConstantShift(shift_type, constant_shift)) => (
            register_file[reg_to_shift as usize],
            constant_shift,
            shift_type,
        ),
        Operand2::ShiftedReg(reg_to_shift, Shift::RegisterShift(shift_type, shift_reg)) => (
            register_file[reg_to_shift as usize],
            (register_file[shift_reg as usize] & mask(8)) as u8,
            shift_type,
        ),
    };

    shift(to_shift, shift_amt, shift_type)
}

pub fn shift(to_shift: u32, shift_amt: u8, shift_type: ShiftType) -> (u32, bool) {
    if shift_amt == 0 {
        return (to_shift, false);
    };
    match shift_type {
        ShiftType::Lsl => to_shift.overflowing_shl(u32::from(shift_amt)),
        ShiftType::Lsr => to_shift.overflowing_shr(u32::from(shift_amt)),
        ShiftType::Asr => {
            let (res, cout) = (to_shift as i32).overflowing_shr(u32::from(shift_amt));
            (res as u32, cout)
        }
        ShiftType::Ror => (
            to_shift.rotate_right(u32::from(shift_amt)),
            extract_bit(&to_shift, shift_amt - 1),
        ),
    }
}

pub fn perform_processing_operation(op1: i32, op2: i32, opcode: ProcessingOpcode) -> (i32, bool) {
    match opcode {
        ProcessingOpcode::And | ProcessingOpcode::Tst => (op1 & op2, false),
        ProcessingOpcode::Eor | ProcessingOpcode::Teq => (op1 ^ op2, false),
        ProcessingOpcode::Sub => op1.overflowing_sub(op2),
        ProcessingOpcode::Rsb => op2.overflowing_sub(op1),
        ProcessingOpcode::Add => op1.overflowing_add(op2),
        ProcessingOpcode::Cmp => (op1 - op2, op1 >= op2),
        ProcessingOpcode::Orr => (op1 | op2, false),
        ProcessingOpcode::Mov => (op2, false),
    }
}

pub fn extract_bit(word: &u32, index: u8) -> bool {
    ((word >> index) & 1) == 1
}

pub fn signed_24_to_32(num: i32) -> i32 {
    if extract_bit(&(num as u32), 23) {
        num | !mask(24) as i32
    } else {
        num
    }
}
