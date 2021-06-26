use super::utils;
use crate::types::*;

pub fn decode(instr: &u32) -> Result<ConditionalInstruction> {
    let mut bytes: &mut [u8];
    utils::to_u8_slice(*instr, bytes);
}
