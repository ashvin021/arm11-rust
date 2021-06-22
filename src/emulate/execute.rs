use crate::types::{Instruction::*, *};

use super::state::*;
use super::{alu, utils};

impl ConditionalInstruction {
    fn satisfies_cpsr(self: &Self, cpsr_contents: &u32) -> bool {
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

fn execute_branch(state: &mut EmulatorState, instr: InstructionBranch) -> Result<()> {
    Ok(())
}

fn execute_transfer(state: &mut EmulatorState, instr: InstructionTransfer) -> Result<()> {
    Ok(())
}

fn execute_processing(state: &mut EmulatorState, instr: InstructionProcessing) -> Result<()> {
    Ok(())
}

fn execute_multiply(state: &mut EmulatorState, instr: InstructionMultiply) -> Result<()> {
    Ok(())
}

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
