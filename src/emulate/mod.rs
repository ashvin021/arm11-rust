mod decode;
mod execute;
mod fetch;
mod state;
mod utils;

use std::fs;

use super::types::*;

pub fn run_pipeline(state: &mut state::EmulatorState) -> Result<()> {
    loop {
        // execute
        if let Some(to_execute) = state.pipeline_mut().decoded {
            // check: is halt?
            if let Instruction::Halt = to_execute.instruction {
                return Ok(());
            }
            // execute otherwise
            execute::execute(state, to_execute)?;
        }

        // decode
        if let Some(word) = state.pipeline_mut().fetched {
            // pipeline.decoded = Some(decode::decode(word)?);
        }

        // fetch
        state.pipeline_mut().fetched = Some(fetch::fetch(state)?);
    }
}

pub fn run(filename: &String) -> Result<()> {
    // Read binary from file
    let bytes: Vec<u8> = fs::read(filename)?;

    // Create emulator and load binary
    let mut emulator = state::EmulatorState::with_memory(bytes);

    // Run emulator
    run_pipeline(&mut emulator)?;
    emulator.print_state();

    Ok(())
}
