use super::state::EmulatorState;
use crate::{
    constants::{BYTES_IN_WORD, PC},
    types::*,
};

pub fn fetch(state: &mut EmulatorState) -> Result<u32> {
    let pc = *state.read_reg(PC);
    state.write_reg(PC, pc + BYTES_IN_WORD as u32);
    state.read_memory(pc as usize)
}
