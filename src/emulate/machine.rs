use std::convert::TryInto;

use super::decode::decode;
use super::utils;
use crate::types::*;

pub struct ArmMachineState {
    main_memory: [u8; 65535],
    register_file: [u32; 17],
    pipeline: Pipeline,
}

pub struct Pipeline {
    fetched: Option<Instruction>,
    decoded: Option<Instruction>,
}

impl ArmMachineState {
    const PC: u8 = 15;
    const CPSR: u8 = 16;

    pub fn new() -> Self {
        let pipeline = Pipeline {
            fetched: None,
            decoded: None,
        };
        ArmMachineState {
            main_memory: [0; 65535],
            register_file: [0; 17],
            pipeline,
        }
    }

    pub fn load_instructions(self: &mut Self, instructions: Vec<u8>) {
        let bytes = &instructions[..];
        for (dst, data) in self.main_memory.iter_mut().zip(bytes.iter()) {
            *dst = *data
        }
    }

    pub fn print_state(self: &Self) {
        println!("Registers:");
        for (index, contents) in self.register_file.iter().enumerate() {
            match index {
                0..=12 => println!("${: <3}: {: >10} (0x{:0>8x})", index, contents, contents),
                15 => println!("PC  : {: >10} (0x{:0>8x})", contents, contents),
                16 => println!("CPSR: {: >10} (0x{:0>8x})", contents, contents),
                _ => (),
            }
        }
        println!("Non-zero memory:");
        for i in (0..65535).step_by(4) {
            if i + 4 >= 65535 {
                continue;
            }
            let bytes: [u8; 4] = self.main_memory[i..i + 4]
                .try_into()
                .expect("slice with incorrect length");
            let word = utils::to_u32_print(&bytes);

            if word == 0 {
                continue;
            }
            println!("0x{:0>8x}: 0x{:0>8x}", i, word);
        }
    }

    pub fn run(self: &mut Self) -> Result<()> {
        loop {
            if let Some(to_execute) = self.pipeline.decoded.clone() {
                if let Instruction::Raw(0) = to_execute {
                    break;
                } else {
                    self.execute(to_execute)?;
                }
            }

            if let Some(fetched) = &self.pipeline.fetched {
                if let Instruction::Raw(word) = fetched {
                    self.pipeline.decoded = Some(decode(word)?);
                }
            }

            self.pipeline.fetched = Some(self.fetch_next()?);
        }

        Ok(())
    }

    fn fetch_next(self: &mut Self) -> Result<Instruction> {
        let from: usize = self.register_file[ArmMachineState::PC as usize].try_into()?;
        let bytes: [u8; 4] = self.main_memory[from..from + 4].try_into()?;
        let word = utils::to_u32_reg(&bytes);
        Ok(Instruction::Raw(word))
    }

    fn execute(self: &mut Self, instr: Instruction) -> Result<()> {
        match instr {
            Instruction::DataProc { .. } => self.execute_dataproc(instr),
            Instruction::Multiply { .. } => self.execute_multiply(instr),
            Instruction::SDT { .. } => self.execute_sdt(instr),
            Instruction::Branch { .. } => self.execute_branch(instr),
            Instruction::Raw(b) => {
                Err(format!("Cannot execute undecoded instruction - 0x{:0>8x}", b).into())
            }
        }
    }

    fn execute_dataproc(self: &mut Self, instr: Instruction) -> Result<()> {
        Ok(())
    }

    fn execute_multiply(self: &mut Self, instr: Instruction) -> Result<()> {
        Ok(())
    }

    fn execute_sdt(self: &mut Self, instr: Instruction) -> Result<()> {
        Ok(())
    }

    fn execute_branch(self: &mut Self, instr: Instruction) -> Result<()> {
        Ok(())
    }
}
