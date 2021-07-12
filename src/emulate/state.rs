use std::convert::TryInto;

use crate::constants::*;
use crate::types::*;

pub struct EmulatorState {
    memory: [u8; MEMORY_SIZE],
    register_file: [u32; NUM_REGS],
    pub pipeline: Pipeline,
}

pub struct Pipeline {
    pub fetched: Option<u32>,
    pub decoded: Option<ConditionalInstruction>,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            fetched: None,
            decoded: None,
        }
    }

    pub fn flush(&mut self) {
        self.fetched = None;
        self.decoded = None;
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl EmulatorState {
    pub fn new() -> Self {
        EmulatorState {
            memory: [0; MEMORY_SIZE],
            register_file: [0; NUM_REGS],
            pipeline: Pipeline::new(),
        }
    }

    pub fn with_memory(mut bytes: Vec<u8>) -> Self {
        bytes.resize(MEMORY_SIZE, 0);
        EmulatorState {
            memory: bytes.try_into().unwrap(),
            register_file: [0; NUM_REGS],
            pipeline: Pipeline::new(),
        }
    }

    pub fn regs(&self) -> &[u32; NUM_REGS] {
        &self.register_file
    }

    // quick ways to read PC and CPSR
    pub fn read_reg(&self, index: usize) -> &u32 {
        &self.register_file[index]
    }

    pub fn write_reg(&mut self, index: usize, val: u32) {
        self.register_file[index] = val;
    }

    pub fn read_memory(&self, address: usize) -> Result<u32> {
        let bytes: [u8; BYTES_IN_WORD] =
            self.memory[address..address + BYTES_IN_WORD].try_into()?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn write_memory(&mut self, address: usize, val: u32) {
        let bytes = val.to_le_bytes();
        self.memory[address..address + BYTES_IN_WORD].clone_from_slice(&bytes[..]);
    }

    pub fn set_flags(&mut self, flag: CpsrFlag, set: bool) {
        if set {
            self.register_file[CPSR] |= 1 << flag as u32;
        } else {
            self.register_file[CPSR] &= !(1 << flag as u32);
        }
    }

    pub fn print_state(&self) {
        println!("Registers:");
        for (index, contents) in self.register_file.iter().enumerate() {
            const MAX_GENERAL_REG: usize = NUM_GENERAL_REGS - 1;
            match index {
                0..=MAX_GENERAL_REG => {
                    println!(
                        "${: <3}: {: >10} (0x{:0>8x})",
                        index, *contents as i32, contents
                    )
                }
                PC => {
                    println!("PC  : {: >10} (0x{:0>8x})", *contents as i32, contents)
                }
                CPSR => {
                    println!("CPSR: {: >10} (0x{:0>8x})", *contents as i32, contents)
                }
                _ => (),
            }
        }
        println!("Non-zero memory:");
        for i in (0..MEMORY_SIZE).step_by(BYTES_IN_WORD) {
            if i + BYTES_IN_WORD >= MEMORY_SIZE {
                continue;
            }
            let bytes: [u8; BYTES_IN_WORD] = self.memory[i..i + BYTES_IN_WORD]
                .try_into()
                .expect("slice with incorrect length");
            let word = i32::from_be_bytes(bytes);

            if word == 0 {
                continue;
            }
            println!("0x{:0>8x}: 0x{:0>8x}", i, word);
        }
    }
}

impl Default for EmulatorState {
    fn default() -> Self {
        Self::new()
    }
}
