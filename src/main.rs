use std::{env, fs, process::{Command, exit}, collections::HashMap};

mod tokenizer;
mod parser;
mod generator;

const VERSION: &'static str = "0.2.0";
const CL_ARGUMENTS: [&str; 12] = ["--help", "--preserve_asm", "--preserve_o", "-b", "-o", "--assemble", "--compile", "--run", "--tokenize", "--parse", "--version", "--silent"];
const HELP: [&str; 27] = [
        "ABOUT", 
        "", 
        "© 2026 Yarema Mushkevych",
        "",
        "This is the Butter Compiler (butterc)", "It can compile butter programs to x86_64 Linux bytecode through nasm and ld", 
        "", 
        "USAGE", 
        "",
        "butterc <FILE> [<FLAGS>]",
        "", 
        "FLAGS", 
        "",
        "--help | print this to the console", 
        "--preserve_asm | preserve all `.asm` files created during the compilation step",
        "--preserve_o | preserve all `.o` files created during the assembly step",
        "-b <directory> | specify the build directory",
        "-o <name> | specify the name of the final output file",
        "   by default this is the name of the input file with the `.btr` extension removed",
        "--assemble | stop once all `.o` files have been generated and do not link into an executable",
        "--compile | stop once all `.asm` files have been generated and do not assemble into `.o` files",
        "--run | (attempt to) run the final executable (if it exists) upon end of compilation",
        "--tokenize | stop at the tokenization step",
        "--parse | stop at the parsing step",
        "--version | list the compiler version",
        "--silent | silence the compiler such that no output will be given",
        "   this only silences success messages, not error messages. Error messages are unsilencable"];
fn main() {
    let arguments: Vec<String> = env::args().collect();  
    if arguments.len() == 1 {
        eprintln!("\x1b[1mCompilerError\x1b[0m: Expected file and/or compiler flags\n\n---\n");
        for line in HELP {
            eprintln!("{line}");
        }
        exit(1);
    }

    if arguments.contains(&String::from("--help")) {
        for line in HELP {
            println!("{line}");
        }
        exit(0);
    }
    if arguments.contains(&String::from("--version")) {
        println!("butterc version {}", VERSION);
        exit(0);
    }

    for argument in &arguments {
        if argument.chars().nth(0).unwrap() == '-' && !CL_ARGUMENTS.contains(&argument.as_str()) {
            eprintln!("\x1b[1mCompilerError\x1b[0m: Unknown command-line argument `{argument}`");
            exit(1);
        }
    }

    if arguments.len() >= 2 && CL_ARGUMENTS.contains(&arguments[1].clone().as_str()) {
        eprintln!("\x1b[1mCompilerError\x1b[0m: Command-line argument `{}` cannot be used without a file", arguments[1]);
        eprintln!("remember that the order of command-line arguments is `butterc <file> [<flags>]`; if you provided a file, ensure that it is in the correct position");
        eprintln!("if you provided a file to link with using the `-l` command-line argument, remember that the compiler still requires a file to use as the \"base\"");
        exit(1);
    }
    let mut extern_modules: Vec<String> = vec![];
    if get_ext(arguments[1].clone()) == "btr".to_string() {
        if !fs::exists(arguments[1].clone()).unwrap() {
            eprintln!("\x1b[1mFileNotFoundError\x1b[0m: No such file `{}`", arguments[1].clone());
            exit(1);
        }
        let mut extern_functions: Vec<HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>> = vec![];
        let mut tokens: Vec<String> = parser::alias(tokenizer::tokenize(arguments[1].clone()), parser::get_aliases(tokenizer::tokenize(arguments[1].clone())));
        if !arguments.contains(&String::from("--tokenize")) {
            extern_modules = parser::get_imports(tokens.clone());
            for extern_module in &extern_modules {
                if !fs::exists(extern_module.clone()).unwrap() {
                    eprintln!("\x1b[1mFileNotFoundError\x1b[0m: No such file `{extern_module}`");
                    exit(1);
                }
                extern_functions.push(parser::preprocess(tokenizer::tokenize(extern_module.clone())));
            }
            let mut unified_extern_functions: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = HashMap::new();
            for function in &extern_functions {
                unified_extern_functions.extend(function.clone());
            }
            let mut module_tokens: Vec<Vec<String>> = vec![];
            let mut module_code: Vec<Vec<Vec<String>>> = vec![];
            let build_dir = if arguments.contains(&String::from("-b")) {
                arguments[arguments.iter().position(|a|a==&"-b".to_string()).unwrap()+1].clone() + if arguments[arguments.iter().position(|a|a==&"-b".to_string()).unwrap()+1].clone().chars()
                .nth(arguments[arguments.iter().position(|a|a==&"-b".to_string()).unwrap()+1].clone().chars().count()) != Some ('/') {"/"} else {""}
            } else {String::from("./")};
            for extern_module in &extern_modules {
                module_tokens.push(parser::alias(tokenizer::tokenize(extern_module.clone()), parser::get_aliases(tokenizer::tokenize(extern_module.clone()))));
                module_tokens.push(tokenizer::tokenize(extern_module.clone()));
                module_code.push(parser::parse(module_tokens[module_tokens.len()-1].clone(), HashMap::new()));
                let injections = parser::get_injections(module_tokens[module_tokens.len()-1].clone());
                for injection in injections {
                    for itoken in injection.into_iter().rev() {
                        tokens.insert(0, itoken.clone());
                    }
                }
            }
            tokens = parser::alias(tokens.clone(), parser::get_aliases(tokens.clone()));
            let code: Vec<Vec<String>> = parser::parse(tokens.clone(), unified_extern_functions);
            if !arguments.contains(&String::from("--parse")) {
                generator::write_asm(code, get_name(arguments[1].clone()), build_dir.clone(), &extern_functions);
                for code in &module_code {
                    generator::write_asm(code.clone(), extern_modules[module_code.iter().position(|c|c==code).unwrap()].clone(), build_dir.clone(), &vec![]);
                }
                if !arguments.contains(&String::from("--compile")) {
                    assemble(get_name(arguments[1].clone()), build_dir.clone(),&extern_modules);
                    if !arguments.contains(&String::from("--assemble")) {
                        link(get_name(arguments[1].clone()), build_dir.clone(), &extern_modules[0..], if arguments.contains(&String::from("-o")) {arguments[arguments.iter().position(|o|o==&"-o".to_string()).unwrap()+1].clone()} else {"".to_string()});
                        if arguments.contains(&String::from("--run")) {
                            let executable = if (if arguments.contains(&String::from("-o")) {arguments[arguments.iter().position(|o|o==&"-o".to_string()).unwrap()+1].clone()} else {"".to_string()}) == "".to_string() {
                                build_dir.clone().as_str().to_owned() + &arguments[1].clone().as_str().to_owned().split('.').collect::<Vec<&str>>()[0].to_string()
                            } else {
                                build_dir.clone().as_str().to_owned() + &if arguments.contains(&String::from("-o")) {arguments[arguments.iter().position(|o|o==&"-o".to_string()).unwrap()+1].clone()} else {"".to_string()}.as_str().to_owned()
                            };
                            match Command::new(executable.clone()).output() {
                                Ok(out) => {
                                    if !arguments.contains(&String::from("--silent")) {
                                        println!("running `{executable}` ... \n");
                                    }
                                    if let Some(o)  = out.status.code() {
                                        if out.stdout.len() > 0 {println!("stdout: {}", String::from_utf8_lossy(&out.stdout));}
                                        if out.stderr.len() > 0 {println!("stderr: {}", String::from_utf8_lossy(&out.stderr));}
                                        if !arguments.contains(&String::from("--silent")) {
                                            println!("\x1b[1mRuntimeSuccess\x1b[0m: Executable ran successfully");
                                        }
                                        if o != 0 {
                                            println!("Executable finished with exit code {:?}", &o); 
                                        } else {
                                            if !arguments.contains(&String::from("--silent")) {
                                                eprintln!("Executable finished with exit code {:?}", &o); 
                                            }
                                        }
                                    } else {
                                        eprintln!("\x1b[1mRuntimeError\x1b[0m: Executable crashed");
                                        exit(1);
                                    }
                                },
                                Err(_) => {
                                    eprintln!("\x1b[1mRuntimeError\x1b[0m: Executable failed to run");
                                    exit(1);
                                },
                            };
                        }
                        
                    } else {
                        let build_dir = if arguments.contains(&String::from("-b")) {
                            arguments[arguments.iter().position(|a|a==&"-b".to_string()).unwrap()+1].clone() + if arguments[arguments.iter().position(|a|a==&"-b".to_string()).unwrap()+1].clone().chars()
                            .nth(arguments[arguments.iter().position(|a|a==&"-b".to_string()).unwrap()+1].clone().chars().count()) != Some ('/') {"/"} else {""}
                        } else {String::from("./")};
                        if !arguments.contains(&"--preserve_asm".to_string()) {
                            match fs::exists(format!("{}{}.asm", build_dir.clone(), get_name(arguments[1].clone()))) {
                                Ok(true) => {
                                    match fs::remove_file(format!("{}{}.asm", build_dir.clone(), get_name(arguments[1].clone()))) {
                                        Ok(_) => {},
                                        Err(_) => {
                                            eprintln!("\x1b[1mCleanupError\x1b[0m: Failed to remove .asm file");
                                        },
                                    }
                                },
                                _ => {
                                    eprintln!("\x1b[1mGenerationError\x1b[0m: Assembly file could not be generated; compilation failed");
                                    exit(1)
                                },
                            }
                        }
                        if !arguments.contains(&String::from("--silent")) {
                            println!("\x1b[1mCompilationSuccess\x1b[0m: Compilation ended at step 4: Assembly");
                            exit(0);
                        }
                    }
                } else {
                    if !arguments.contains(&String::from("--silent")) {
                        println!("\x1b[1mCompilationSuccess\x1b[0m: Compilation ended at step 3: Compilation");
                        exit(0);
                    }
                }
            } else {
                if !arguments.contains(&String::from("--silent")) {
                    println!("\x1b[1mCompilationSuccess\x1b[0m: Compilation ended at step 2: Parsing");
                    exit(0);
                }
            }
        } else {
            if !arguments.contains(&String::from("--silent")) {
                println!("\x1b[1mCompilationSuccess\x1b[0m: Compilation ended at step 1: Tokenization");
                println!("\n");
            }
            exit(0);
        }
    } else {
        eprintln!("\x1b[1mCompilerError\x1b[0m: Incompatible File Extension\nExpected `.btr`");
        eprintln!("For proper usage, run `butterc <file>.btr`");
        exit(1);    
    }

    let build_dir = if arguments.contains(&String::from("-b")) {
        arguments[arguments.iter().position(|a|a==&"-b".to_string()).unwrap()+1].clone() + if arguments[arguments.iter().position(|a|a==&"-b".to_string()).unwrap()+1].clone().chars()
        .nth(arguments[arguments.iter().position(|a|a==&"-b".to_string()).unwrap()+1].clone().chars().count()) != Some ('/') {"/"} else {""}
    } else {String::from("./")};
    
    let mut dirs: Vec<String> = extern_modules.iter().map(|m|if m.contains('/') {if let Some(index) = m.rfind('/') {m[index+1..].to_string()} else {m.to_string()}} else {m.to_string()}).collect::<Vec<String>>();
    dirs.push(get_name(arguments[1].clone()));
    
    if !arguments.contains(&"--preserve_asm".to_string()) {
        for dir in &dirs {
            match fs::exists(format!("{}{}.asm", build_dir.clone(), dir)) {
            Ok(true) => {
                match fs::remove_file(format!("{}{}.asm", build_dir.clone(), dir)) {
                    Ok(_) => {},
                    Err(_) => {
                        eprintln!("\x1b[1mCleanupError\x1b[0m: Failed to remove .asm file");
                    },
                }
            },
            _ => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Assembly file could not be generated; compilation failed");
                exit(1)
            },
        }
        }
    }
    if !arguments.contains(&"--preserve_o".to_string()) {
        for dir in &dirs {
            match fs::exists(format!("{}{}.o", build_dir.clone(), dir)) {
            Ok(true) => {
                match fs::remove_file(format!("{}{}.o", build_dir.clone(), dir)) {
                    Ok(_) => {},
                    Err(_) => {
                        eprintln!("\x1b[1mCleanupError\x1b[0m: Failed to remove .o file");
                    },
                }
            },
            _ => {
                eprintln!("\x1b[1mAssemblyError\x1b[0m: Assembly file could not be assembled; compilation failed");
                eprintln!("This is an issue with the compiler, not your code. Please report this issue ASAP");
                exit(1)
            },
        }
        }
    }
    let executable = if (if arguments.contains(&String::from("-o")) {arguments[arguments.iter().position(|o|o==&"-o".to_string()).unwrap()+1].clone()} else {"".to_string()}) == "".to_string() {
        build_dir.clone().as_str().to_owned() + &arguments[1].clone().as_str().to_owned().split('.').collect::<Vec<&str>>()[0].to_string()
    } else {
        build_dir.clone().as_str().to_owned() + &if arguments.contains(&String::from("-o")) {arguments[arguments.iter().position(|o|o==&"-o".to_string()).unwrap()+1].clone()} else {"".to_string()}.as_str().to_owned()
    };
    if !arguments.contains(&String::from("--run")) {
        if !arguments.contains(&String::from("--silent")) {
            println!("\x1b[1mCompilationSuccess\x1b[0m: Compilation ended at step 5: Linking");
            println!("Created executable `{}`", executable);
            println!("All steps completed successfully");
        }
    } else {
        if !arguments.contains(&String::from("--silent")) {
            println!("\nAll steps completed successfully");
        }    
    }
    exit(0);
}

fn get_ext(file: String) -> String {
    let split: Vec<&str> = file.split('/').collect();
    let fname = split[split.len()-1];
    let name: Vec<&str> = fname.split('.').collect();
    if name.len() == 1 {
        eprintln!("\x1b[1mCompilerError\x1b[0m: No extension found on file `{}`", name.concat());
        eprintln!("expected `{}.btr`", name.concat());
        exit(1);
    }
    return name[1].to_string();
}

fn get_name(file: String) -> String{
    let split: Vec<&str> = file.split('/').collect();
    return split[split.len()-1].to_string();
}

fn assemble(name: String, build: String, modules:&[String]) {
    let asm_name = build.as_str().to_owned() + &name.as_str().to_owned() + ".asm";
    let o_name = build.as_str().to_owned() + &name.as_str().to_owned() + ".o";
    match Command::new("nasm").args(&["-f", "elf64", &asm_name, "-o", &o_name]).output() {
        Ok(_) => {},
        Err(_) => {
            eprintln!("\x1b[1mAssemblyError\x1b[0m: Failed to assemble");
            eprintln!("Ensure nasm is installed");
            exit(1);
        },
    }

    for module in modules {
        let mut lmodule = module.clone();
        if lmodule.contains('/') {
            if let Some(index) = module.rfind('/') {
                lmodule = lmodule[index+1..].to_string();
            }
        }
        match Command::new("nasm").args(&["-f", "elf64", &format!("{build}{lmodule}.asm"), "-o", &format!("{build}{lmodule}.o")]).output() {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mAssemblyError\x1b[0m: Failed to assemble");
                eprintln!("Ensure nasm is installed");
                exit(1);
            },
        }
    }

}

fn link(name: String, build:String, modules:&[String], bin_name: String) {
    let o_name = build.as_str().to_owned() + &name.as_str().to_owned() + ".o";
    let bin_name: String = if bin_name == "".to_string() {
        build.as_str().to_owned() + &name.as_str().to_owned().split('.').collect::<Vec<&str>>()[0].to_string()
    } else {
        build.as_str().to_owned() + &bin_name.as_str().to_owned()
    };
    let mut args:Vec<String>=vec![];
    args.push(o_name.clone());
    for module in modules {
        let mut lmodule = module.clone();
        if lmodule.contains('/') {
            if let Some(index) = module.rfind('/') {
                lmodule = lmodule[index+1..].to_string();
            }
        }
        args.push(format!("{build}{}.o", lmodule.clone()));
    }
    args.push(String::from("-o"));
    args.push(bin_name.clone());
    match Command::new("ld").args(args.as_slice()).output() {
        Ok(_) => {},
        Err(_) => {
            eprintln!("\x1b[1mLinkError\x1b[0m: Failed to link");
        },
    }
}
