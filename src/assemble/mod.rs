mod encode;
mod parse;

use std::{collections::HashMap, fs, io::Write, rc::Rc};

use super::{constants::*, types::*};

pub fn run(input_filename: &str, output_filename: &str) -> Result<()> {
    let raw = fs::read_to_string(input_filename)?;

    // First pass - populate symbol table and isntructions list
    let (symbol_table, instructions) = extract_labels_and_instructions(raw);

    let rc_symbol_table = Rc::new(symbol_table);
    let mut assembled = Vec::new();
    let mut additional = Vec::new();
    let mut next_free_address = instructions.len() * BYTES_IN_WORD;

    // Second pass, parse the strings and add them to vectors
    for (current_address, instr) in instructions.iter().enumerate() {
        let st = rc_symbol_table.clone();
        let (parsed, opt_data) = parse::parse_asm(
            instr.as_str(),
            current_address * BYTES_IN_WORD,
            next_free_address,
            st,
        )?;

        let encoded = encode::encode(parsed);
        assembled.extend_from_slice(&encoded.to_le_bytes());

        if let Some(data) = opt_data {
            additional.extend_from_slice(&data.to_le_bytes());
            next_free_address += BYTES_IN_WORD;
        }
    }

    // Add additional data to the end of byte vector and write all to the output file
    assembled.append(&mut additional);
    let mut file = fs::File::create(output_filename)?;
    file.write_all(&assembled)?;

    Ok(())
}

fn extract_labels_and_instructions(raw: String) -> (HashMap<String, u32>, Vec<String>) {
    let mut symbol_table = HashMap::new();
    let mut instructions = Vec::new();

    let mut address = 0;
    for line in raw.lines() {
        let len = line.len();

        // If the line is empty continue
        if len == 0 {
            continue;
        }

        // If the line ends with ":" it is a label, else it is an instruction
        if &line[len - 1..] == ":" {
            symbol_table.insert(String::from(&line[..len - 1]), address);
        } else {
            instructions.push(String::from(line));
            address += BYTES_IN_WORD as u32;
        }
    }

    (symbol_table, instructions)
}
