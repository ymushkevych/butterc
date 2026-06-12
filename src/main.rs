use std::{env, process::{Command, exit}, fs};
mod tokenizer;
mod parser;
mod generator;


fn main() {
    let arguments: Vec<String> = env::args().collect();  
    if arguments.len() == 1 {
        eprintln!("\x1b[1mCompilationError\x1b[0m: Expected file");
        exit(1);
    }
    if get_ext(arguments[1].clone()) == "btr".to_string() {
        if !fs::exists(arguments[1].clone()).unwrap() {
            eprintln!("\x1b[1mFileNotFoundError\x1b[0m: No such file `{}`", arguments[1].clone());
            exit(1);
        }
        let tokens: Vec<String> = tokenizer::tokenize(arguments[1].clone(), &arguments[0..]);
        let code: Vec<Vec<String>> = parser::parse(tokens,false, vec![]);
        generator::write_asm(code, get_name(arguments[1].clone()), true, true);
        assemble(get_name(arguments[1].clone()));
        link(get_name(arguments[1].clone()));
    } else {
        eprintln!("\x1b[1mCompilationError\x1b[0m: Incompatible File Extension\nExpected `.btr``");
        eprintln!("For proper usage, run `butterc <file>.btr`");
        exit(1);    
    }

    if !arguments.contains(&"--preserve_asm".to_string()) {
        match fs::exists(format!("./{}.asm", get_name(arguments[1].clone()))) {
            Ok(true) => {
                match fs::remove_file(format!("./{}.asm", get_name(arguments[1].clone()))) {
                    Ok(_) => {},
                    Err(_) => {
                        eprintln!("\x1b[1mCleanupError\x1b[0m: Failed to remove .asm file");
                    },
                }
            },
            _ => {
                eprintln!("\x1b[1mFileNotFoundError\x1b[0m: No assembly file found to remove");
                exit(1)
            },
        }
    }

    if !arguments.contains(&"--preserve_o".to_string()) {
        match fs::exists(format!("./{}.o", get_name(arguments[1].clone()))) {
            Ok(true) => {
                match fs::remove_file(format!("./{}.o", get_name(arguments[1].clone()))) {
                    Ok(_) => {},
                    Err(_) => {
                        eprintln!("\x1b[1mCleanupError\x1b[0m: Failed to remove .o file");
                    },
                }
            },
            _ => {
                eprintln!("\x1b[1mFileNotFoundError\x1b[0m: No object file found to remove");
                exit(1)
            },
        }
    }
}


fn get_ext(file: String) -> String {
    let split: Vec<&str> = file.split('/').collect();
    let fname = split[split.len()-1];
    let name: Vec<&str> = fname.split('.').collect();
    if name.len() == 1 {
        eprintln!("\x1b[1mCompilationError\x1b[0m: No extension found on file `{}`", name.concat());
        exit(1);
    }
    return name[1].to_string();
}

fn get_name(file: String) -> String{
    let split: Vec<&str> = file.split('/').collect();
    return split[split.len()-1].to_string();
}

fn assemble(name: String) {
    let asm_name = name.as_str().to_owned() + ".asm";
    let o_name = name.as_str().to_owned() + ".o";
    match Command::new("nasm").args(&["-f", "elf64", &asm_name, "-o", &o_name]).output() {
        Ok(_) => {},
        Err(_) => {
            eprintln!("\x1b[1mCompilationError\x1b[0m: Failed to assemble");
            eprintln!("Ensure nasm is installed");
            exit(1);
        },
    }
}

fn link(name: String) {
    let o_name = name.as_str().to_owned() + ".o";
    let bin_name = name.as_str().to_owned().split('.').collect::<Vec<&str>>()[0].to_string();
    match Command::new("ld").args(&[&o_name, "-o", &bin_name]).output() {
        Ok(_) => {},
        Err(_) => {
            eprintln!("\x1b[1mCompilationError\x1b[0m: Failed to link");
        },
    }
}
