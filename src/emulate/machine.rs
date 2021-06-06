use std::convert::TryInto;

use super::utils;
use crate::types::Instruction;

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
    pub fn new() -> ArmMachineState {
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

    pub fn print_state(self: Self) {
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
}
