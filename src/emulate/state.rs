use std::convert::TryInto;

use crate::constants::*;
use crate::types::*;

use super::utils;

pub struct EmulatorState {
    pub memory: [u8; MEMORY_SIZE],
    register_file: [u32; NUM_REGS],
    pipeline: Pipeline,
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

impl EmulatorState {
    pub const PC: usize = 15;
    pub const CPSR: usize = 16;

    pub fn new() -> Self {
        EmulatorState {
            memory: [0; MEMORY_SIZE],
            register_file: [0; NUM_REGS],
            pipeline: Pipeline::new(),
        }
    }

    pub fn with_memory(bytes: Vec<u8>) -> Self {
        bytes.resize(MEMORY_SIZE, 0);
        EmulatorState {
            memory: bytes.try_into().unwrap(),
            register_file: [0; NUM_REGS],
            pipeline: Pipeline::new(),
        }
    }

    pub fn pipeline_mut(self: &Self) -> &mut Pipeline {
        &mut self.pipeline
    }

    // quick ways to read PC and CPSR
    pub fn reg(self: &Self, index: usize) -> &u32 {
        &self.register_file[index]
    }

    pub fn reg_mut(self: &Self, index: usize) -> &mut u32 {
        &mut self.register_file[index]
    }

    pub fn read_memory(self: &mut Self, address: u32) -> &u32 {
        &0
    }

    pub fn write_memory(self: &mut Self, address: u32, val: u32) {}

    pub fn set_flags(self: &mut Self, flag: CpsrFlag, set: bool) {
        let cpsr_contents = self.reg_mut(EmulatorState::CPSR);
        if set {
            *cpsr_contents |= 1 << flag as u32;
        } else {
            *cpsr_contents &= !(1 << flag as u32);
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
            let bytes: [u8; 4] = self.memory[i..i + 4]
                .try_into()
                .expect("slice with incorrect length");
            let word = utils::to_u32_print(&bytes);

            if word == 0 {
                continue;
            }
            println!("0x{:0>8x}: 0x{:0>8x}", i, word);
        }
    }
}
