
use crate::types::*;
use crate::constants::*;

pub struct EmulatorState {
    pub memory: [u8; 65535],
    register_file: [u32; 17],
    pipeline: Pipeline,
}

pub struct Pipeline {
    fetched: Option<u32>,
    decoded: Option<Instruction>,
}

impl Pipeline {
    pub fn flush(&mut self) {
        self.fetched = None;
        self.decoded = None;
    }
}

impl EmulatorState {
    const PC: usize = 15;
    const CPSR: usize = 16;
    const MEM_SIZE: usize = 65535;

    pub fn new() -> Self {
        let pipeline = Pipeline {
            fetched: None,
            decoded: None,
        };

        EmulatorState {
            memory: [0; 65535],
            register_file: [0; 17],
            pipeline,
        }
    }

    pub fn load_instructions(self: &mut Self, bytes: Vec<u8>) {
        
    }

    // maybe: quick ways to read PC and CPSR
    // 

    pub fn read_memory(self: &mut Self, address: u32) -> u32 {

    }

    pub fn write_memory(self: &mut Self, address: u32, val: u32) {

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