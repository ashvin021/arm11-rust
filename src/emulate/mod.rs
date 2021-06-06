mod machine;
mod utils;

use std::{error, fs};

use super::types::*;

pub fn run(filename: &String) -> Result<(), Box<dyn error::Error>> {
    let mut arm11_emulator = machine::ArmMachineState::new();
    let instructions: Vec<u8> = fs::read(filename)?;
    arm11_emulator.load_instructions(instructions);
    arm11_emulator.print_state();

    Ok(())
}
