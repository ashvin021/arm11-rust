mod alu;
mod decode;
mod execute;
mod machine;
mod state;
mod utils;

use std::fs::File;
use std::io::prelude::*;

use super::types::*;

pub fn run_pipeline(state: &mut state::EmulatorState) -> Result<()> {
    let mut pipeline = state.pipeline_mut();
    loop {
        // execute
        if let Some(to_execute) = pipeline.decoded {
            // check: is halt?
            if let Instruction::Halt = to_execute.instruction {
                return Ok(());
            }
            // execute otherwise
            execute::execute(state, to_execute)?;
        }

        // decode
        if let Some(word) = pipeline.fetched {
            pipeline.decoded = Some(decode::decode(word)?);
        }

        // fetch
        pipeline.fetched = Some(fetch_next(state)?);
    }
}

pub fn run(filename: &String) -> Result<()> {
    let mut emulator = state::EmulatorState::new();

    // open file
    let mut file: File = File::open(filename)?;
    // read file
    file.read_to_end(&mut emulator.memory.into())?;

    run_pipeline(&mut emulator)?;

    emulator.print_state();
    Ok(())
}
