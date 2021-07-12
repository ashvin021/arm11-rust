use std::{env, process};

use arm11::assemble;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        3 => {
            let input_filename = &args[1];
            let output_filename = &args[2];
            if let Err(e) = assemble::run(input_filename, output_filename) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }

        _ => {
            println!("Usage: assemble [source] [output]");
            process::exit(1);
        }
    }
}
