mod alu;
mod decode;
mod machine;
mod utils;

use std::fs;

use super::types::*;

pub fn run(filename: &String) -> Result<()> {
    let mut emulator = machine::ArmMachineState::new();
    let instructions: Vec<u8> = fs::read(filename)?;

    emulator.load_instructions(instructions);
    emulator.run()?;
    emulator.print_state();

    Ok(())
}
