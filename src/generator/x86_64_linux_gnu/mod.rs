pub mod nasm

use std::{fs, process::exit};

pub fn determine_assembler() -> String {
    match fs::exists("/usr/bin/nasm") {
        Ok(true) => {return "nasm".to_string();},
        _ => {
            eprintln!("\x1b[1mCompilationError\x1b[0m: No valid assembler found on system");
            eprintln!("Could not find path `/usr/bin/nasm`");
            exit(1);
        },
    }
}
