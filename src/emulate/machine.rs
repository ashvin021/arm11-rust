use super::decode::decode;
use super::utils;
use crate::types::*;

impl ArmMachineState {
    pub fn run(self: &mut Self) -> Result<()> {
        loop {
            // execute
            if let Some(to_execute) = self.pipeline.decoded {
                // check: is halt?
                if let Instruction::Halt = to_execute {
                    break;
                }
                // execute otherwise
                self.execute(to_execute)?;
            }

            // decode
            if let Some(word) = &self.pipeline.fetched {
                self.pipeline.decoded = Some(decode(word)?);
            }

            // fetch
            self.pipeline.fetched = Some(self.fetch_next()?);
        }

        Ok(())
    }

    fn fetch_next(self: &mut Self) -> Result<u32> {
        let from: usize = self.register_file[ArmMachineState::PC].try_into()?;
        let bytes: [u8; 4] = self.main_memory[from..from + 4].try_into()?;
        let word: u32 = utils::to_u32_reg(&bytes);
        Ok(word)
    }
}
