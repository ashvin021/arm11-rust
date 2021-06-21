mod alu;
mod decode;
mod machine;
mod state;
mod utils;

use std::io::prelude::*;
use std::fs::File;

use super::types::*;

pub fn run_pipeline(state: &mut state::EmulatorState) -> Result<()> {
    Ok(())
}

pub fn run(filename: &String) -> Result<()> {
    let mut emulator = state::EmulatorState::new();

    // open file
    let mut file: File = File::open(filename)?;
    // read file
    file.read_to_end(&mut emulator.memory)?;
    

    run_pipeline(emulator)?;
    emulator.print_state();

    Ok(())
}
