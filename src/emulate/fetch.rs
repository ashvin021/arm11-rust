use super::state::*;
use crate::types::*;

pub fn fetch(state: &mut EmulatorState) -> Result<u32> {
    let pc = *state.read_reg(EmulatorState::PC);
    state.write_reg(EmulatorState::PC, pc + 4);
    state.read_memory(pc as usize)
}
