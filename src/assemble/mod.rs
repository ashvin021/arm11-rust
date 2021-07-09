mod parse;

use std::{collections::HashMap, fs};

use super::types::*;

pub fn run(input_filename: &str, output_filename: &str) -> Result<()> {
    let raw = fs::read_to_string(input_filename)?;

    // First pass - populate symbol table and isntructions list
    let (symbol_table, instructions) = extract_labels_and_instructions(raw);

    println!("{:?}", symbol_table);
    println!("{:?}", instructions);

    Ok(())
}

fn extract_labels_and_instructions(raw: String) -> (HashMap<String, u32>, Vec<String>) {
    let mut symbol_table = HashMap::new();
    let mut instructions = Vec::new();

    let mut address = 0;
    for line in raw.lines() {
        let len = line.len();
        if &line[len - 1..] == ":" {
            symbol_table.insert(String::from(&line[..len - 1]), address);
        } else {
            instructions.push(String::from(line));
            address += 4;
        }
    }

    (symbol_table, instructions)
}
