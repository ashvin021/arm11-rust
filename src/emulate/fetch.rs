use super::state::*;
use crate::types::*;
use std::convert::TryInto;

pub fn fetch(state: &mut EmulatorState) -> Result<u32> {
    let from: usize = (*state.read_reg(EmulatorState::PC)).try_into()?;
    state.read_memory(from)
}
