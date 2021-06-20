use std::convert::TryInto;

use super::decode::decode;
use super::{alu, utils};
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
    const PC: usize = 15;
    const CPSR: usize = 16;
    const MEM_SIZE: usize = 65535;

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
        let word: u32 = utils::to_u32_reg(&bytes);
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
        match instr {
            Instruction::DataProc {
                cond,
                opcode,
                set_cond,
                rn,
                rd,
                operand2,
            } if alu::satisfies_cpsr(&cond, &self.register_file[ArmMachineState::CPSR]) => {
                let operand1 = &self.register_file[rn as usize];
                let (operand2, bs_carry_out) = alu::barrel_shifter(operand2, &self.register_file);
                let (result, carry_out) =
                    alu::perform_dataproc_operation(*operand1 as i32, operand2 as i32, opcode);

                match opcode {
                    DataProcOpcode::Cmp | DataProcOpcode::Teq | DataProcOpcode::Tst => (),
                    _ => {
                        self.register_file[rd as usize] = result as u32;
                    }
                }

                if set_cond {
                    let c_flag = !bs_carry_out & carry_out | carry_out;
                    let cpsr_contents = &mut self.register_file[ArmMachineState::CPSR];
                    alu::set_flags(CpsrFlag::CFlag, c_flag, cpsr_contents);
                    alu::set_flags(
                        CpsrFlag::NFlag,
                        utils::extract_bit(&(result as u32), CpsrFlag::NFlag as u8),
                        cpsr_contents,
                    );
                    alu::set_flags(CpsrFlag::ZFlag, result == 0, cpsr_contents);
                }

                Ok(())
            }
            Instruction::DataProc { .. } => Ok(()),
            _ => Err(format!("Cannot execute {:?} as DataProc", instr).into()),
        }
    }

    fn execute_multiply(self: &mut Self, instr: Instruction) -> Result<()> {
        match instr {
            Instruction::Multiply {
                cond,
                accumulate,
                set_cond,
                rd,
                rn,
                rs,
                rm,
            } if alu::satisfies_cpsr(&cond, &self.register_file[ArmMachineState::CPSR]) => {
                let mut result: u32 =
                    &self.register_file[rm as usize] * &self.register_file[rs as usize];

                if accumulate {
                    result += &self.register_file[rn as usize];
                }

                self.register_file[rd as usize] = result;

                if set_cond {
                    let cpsr_contents = &mut self.register_file[ArmMachineState::CPSR];
                    alu::set_flags(
                        CpsrFlag::NFlag,
                        utils::extract_bit(&result, CpsrFlag::NFlag as u8),
                        cpsr_contents,
                    );
                    alu::set_flags(CpsrFlag::ZFlag, result == 0, cpsr_contents);
                }

                Ok(())
            }
            Instruction::Multiply { .. } => Ok(()),
            _ => Err(format!("Cannot execute {:?} as Multiply", instr).into()),
        }
    }

    fn execute_sdt(self: &mut Self, instr: Instruction) -> Result<()> {
        match instr {
            Instruction::SDT {
                cond,
                is_preindexed,
                up_bit,
                load,
                rn,
                rd,
                offset,
            } if alu::satisfies_cpsr(&cond, &self.register_file[ArmMachineState::CPSR]) => {
                let interpreted_offset: i32 = match offset {
                    Operand2::ConstantShift(rotate, imm) => i32::from(rotate) << 8 | i32::from(imm),
                    _ => alu::barrel_shifter(offset, &mut self.register_file).0 as i32,
                };

                let mut mem_address: usize = self.register_file[rn as usize] as usize;

                if is_preindexed {
                    mem_address += if up_bit {
                        interpreted_offset
                    } else {
                        -1 * interpreted_offset
                    } as usize;
                }

                if mem_address <= ArmMachineState::MEM_SIZE {
                    if load {
                        // Load the memory to R[rd], after converting it to u32
                        self.register_file[rd as usize] = utils::to_u32_reg(
                            self.main_memory[mem_address..mem_address + 4].try_into()?,
                        );
                    } else {
                        // Stores the value at Mem[rd] after converting it to u8 slice
                        utils::to_u8_slice(
                            self.register_file[rd as usize],
                            &mut self.main_memory[mem_address..mem_address + 4],
                        );
                    }
                } else {
                    println!(
                        "Error: Out of bounds memory access at 0x{:0>8x}",
                        mem_address
                    );
                }

                if !is_preindexed {
                    self.register_file[rn as usize] += if up_bit {
                        interpreted_offset
                    } else {
                        -1 * interpreted_offset
                    } as u32;
                }

                Ok(())
            }
            Instruction::SDT { .. } => Ok(()),
            _ => Err(format!("Cannot execute {:?} as SDT", instr).into()),
        }
    }

    fn execute_branch(self: &mut Self, instr: Instruction) -> Result<()> {
        match instr {
            Instruction::Branch { cond, offset }
                if alu::satisfies_cpsr(&cond, &self.register_file[ArmMachineState::CPSR]) =>
            {
                self.register_file[ArmMachineState::PC] =
                    ((self.register_file[ArmMachineState::PC] as i32)
                        + utils::signed_24_to_32(offset)) as u32;
                self.pipeline.flush();
                Ok(())
            }
            Instruction::Branch { .. } => Ok(()),
            _ => Err(format!("Cannot execute {:?} as Branch", instr).into()),
        }
    }
}

impl Pipeline {
    pub fn flush(&mut self) {
        self.fetched = None;
        self.decoded = None;
    }
}
