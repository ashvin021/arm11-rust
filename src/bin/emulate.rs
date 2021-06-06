use std::{env, process};

use arm11::emulate;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let filename = &args[1];
            if let Err(e) = emulate::run(filename) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }

        _ => {
            println!("Usage: emulate [binary]");
            process::exit(1);
        }
    }
}
