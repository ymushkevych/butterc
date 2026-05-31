use std::{collections::HashMap, fs::File, io::{BufWriter, Write}, process::exit};

fn is_fnc_call(expr: &String) -> bool {
    if expr.contains("fcall") {
        return true;
    }
    return false;
}

fn is_int_lit(tok: &String) -> bool {
    for _char in tok.chars() {
        if !_char.is_numeric() && _char != '0' {
            return false;
        }
    }
    return true;
}

fn is_bin_expr(expr: &String) -> bool {
    if expr.contains(&"PLUS".to_string()) 
    || expr.contains(&"TIMES".to_string()) 
    || expr.contains(&"MIN".to_string())
    || expr.contains(&"DIV".to_string()) {
        return true;
    } else {        
        return false;
    }
}

fn has_bin_expr(exprs: &[String]) -> bool {
    for expr in exprs {
        if is_bin_expr(expr) {
            continue;
        } else {
            return false;
        }
    }
    return true;
}


fn get_constant_len(constant: &String) -> i32 {
    let mut size: i32 = 0;
    for _ch in constant.chars() {
        size += 1;
    }
    return size;
}

fn get_stack_height(stack: &HashMap<String, Vec<String>>) -> i32 {
    let mut count: i32 = -1;
    for var in stack.keys() {
        let off = stack.get(var).unwrap();
        if off.len() == 0 {
            continue;
        } else if off[0].parse::<i32>().unwrap() > count { 
            count = off[0].parse::<i32>().unwrap();
        }
    }   
    return count;
}

fn move_to_stack(reg: &String, val: &String, mut asm: Vec<String>) -> Vec<String>  {
    asm.push("    mov ".to_string() + reg + ", " + val);
    asm.push("    push ".to_string() + reg);
    
    return asm;
}


fn gen_constant(name : &String, _type: &String, value: &[String], mut asm: Vec<String>) -> Vec<String> {    
    if _type == "int" {
        if has_bin_expr(value) {
            asm = gen_const_bin_expr(value, asm);
        } else {
            asm.push(format!("{name} dq {}", value.concat()));
        }
    } else if _type == "str" {
        asm.push(format!("{name} db {}, 0", value.concat()));
    } else {
        exit(11);
    }

    return asm;
}

fn gen_print_e(prf: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    let mut stmt: Vec<String> = vec![];
    for term in prf {
        asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
        if term.chars().nth(0) == Some('"') {
            for i in 1..term.len()-1 {
                let ch = term.chars().nth(i).unwrap();
                stmt.insert(0, format!("('{}' << {})", ch, (i-1) * 8));
                if i == term.len()-2 {
                    asm = move_to_stack(&"rax".to_string(), &stmt.join("+"), asm);
                    stmt.clear();
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 2".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", i+1));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 16".to_string());
                }
            }
        } else {
            if is_bin_expr(term) {
                asm = gen_bin_expr(&[term.to_owned()], vars, curr_func, true, asm);
                asm.push("    pop rax".to_string());
                asm.push(format!("    add rax, 48"));
                asm.push(format!("    mov rbx, ('' << {})", 8));
                asm.push(format!("    or rax, rbx"));
                asm.push("    push rax".to_string());
                asm.push("    mov rax, 1".to_string());
                asm.push("    mov rdi, 2".to_string());
                asm.push("    mov rsi, rsp".to_string());
                asm.push(format!("    mov rdx, {}", 2));
                asm.push("    syscall".to_string());
                asm.push("    add rsp, 16".to_string());
            } else if is_fnc_call(term){
                let term: Vec<&str> = term.split_ascii_whitespace().collect();
                let term: Vec<String> = term.iter().map(|&a| a.to_string()).collect();
                asm = gen_fnc_call(&term[1],&term[2..], vars, curr_func, true,asm);
                asm.push(format!("    add rax, 48"));
                asm.push(format!("    mov rbx, ('' << {})", 8));
                asm.push(format!("    or rax, rbx"));
                asm.push("    push rax".to_string());
                asm.push("    mov rax, 1".to_string());
                asm.push("    mov rdi, 2".to_string());
                asm.push("    mov rsi, rsp".to_string());
                asm.push(format!("    mov rdx, {}", 2));
                asm.push("    syscall".to_string());
                asm.push("    add rsp, 16".to_string());
            } else {
                if vars.get(curr_func).unwrap().get("stack").unwrap().contains_key(term)
                {
                    let mut offset = vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().get(term).unwrap()[0].parse::<i32>().unwrap();
                    offset = (get_stack_height(vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap())-offset)*8;
                    if curr_func != "" {
                        if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(term) {
                            offset += 8;
                        }
                    }
                    asm.push(format!("    mov rax, [rsp + {}]", offset+8));
                    asm.push(format!("    add rax, 48"));
                    asm.push(format!("    mov rbx, ('' << {})", 8));
                    asm.push(format!("    or rax, rbx"));
                    asm.push("    push rax".to_string());
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 2".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", 2));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 16".to_string());
                } else if vars.get("consts").unwrap().get("int").unwrap().contains_key(term) {
                    asm.push(format!("    mov rax, [{}]", term));
                    asm.push(format!("    add rax, 48"));
                    asm.push(format!("    mov rbx, ('' << {})", 8));
                    asm.push(format!("    or rax, rbx"));
                    asm.push("    push rax".to_string());
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 2".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", 2));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 16".to_string());
                } else {
                    eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable, `{}`", term);
                    exit(1);
                }
            }
            
        }
    }
    asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
    asm = move_to_stack(&"rax".to_string(), &format!("(10 << 8) + ('' << 0)"), asm);
    asm.push("    mov rax, 1".to_string());
    asm.push("    mov rdi, 2".to_string());
    asm.push("    mov rsi, rsp".to_string());
    asm.push(format!("    mov rdx, {}", 2));
    asm.push("    syscall".to_string());
    asm.push("    add rsp, 16".to_string());
    return asm;
}

fn gen_print_f(prf: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    let mut stmt: Vec<String> = vec![];
    for term in prf {
        asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
        if term.chars().nth(0) == Some('"') {
            for i in 1..term.len()-1 {
                let ch = term.chars().nth(i).unwrap();
                stmt.insert(0, format!("('{}' << {})", ch, (i-1) * 8));
                if i == term.len()-2 {
                    asm = move_to_stack(&"rax".to_string(), &stmt.join("+"), asm);
                    stmt.clear();
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 1".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", i+1));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 16".to_string());
                }
            }
        } else {
            if is_bin_expr(term) {
                asm = gen_bin_expr(&[term.to_owned()], vars, curr_func, true, asm);
                asm.push("    pop rax".to_string());
                asm.push(format!("    add rax, 48"));
                asm.push(format!("    mov rbx, ('' << {})", 8));
                asm.push(format!("    or rax, rbx"));
                asm.push("    push rax".to_string());
                asm.push("    mov rax, 1".to_string());
                asm.push("    mov rdi, 1".to_string());
                asm.push("    mov rsi, rsp".to_string());
                asm.push(format!("    mov rdx, {}", 2));
                asm.push("    syscall".to_string());
                asm.push("    add rsp, 16".to_string());
            } else if is_fnc_call(term){
                let term: Vec<&str> = term.split_ascii_whitespace().collect();
                let term: Vec<String> = term.iter().map(|&a| a.to_string()).collect();
                asm = gen_fnc_call(&term[1],&term[2..], vars, curr_func, true,asm);
                asm.push(format!("    add rax, 48"));
                asm.push(format!("    mov rbx, ('' << {})", 8));
                asm.push(format!("    or rax, rbx"));
                asm.push("    push rax".to_string());
                asm.push("    mov rax, 1".to_string());
                asm.push("    mov rdi, 1".to_string());
                asm.push("    mov rsi, rsp".to_string());
                asm.push(format!("    mov rdx, {}", 2));
                asm.push("    syscall".to_string());
                asm.push("    add rsp, 16".to_string());
            } else {
                if vars.get(curr_func).unwrap().get("stack").unwrap().contains_key(term)
                {
                    let mut offset = vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().get(term).unwrap()[0].parse::<i32>().unwrap();
                    offset = (get_stack_height(vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap())-offset)*8;
                    if curr_func != "" {
                        if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(term) {
                            offset += 8;
                        }
                    }
                    asm.push(format!("    mov rax, [rsp + {}]", offset+8));
                    asm.push(format!("    add rax, 48"));
                    asm.push(format!("    mov rbx, ('' << {})", 8));
                    asm.push(format!("    or rax, rbx"));
                    asm.push("    push rax".to_string());
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 1".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", 2));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 16".to_string());
                } else if vars.get("const").unwrap().get("int").unwrap().contains_key(term)  {
                    asm.push(format!("    mov rax, [{}]", term));
                    asm.push(format!("    add rax, 48"));
                    asm.push(format!("    mov rbx, ('' << {})", 8));
                    asm.push(format!("    or rax, rbx"));
                    asm.push("    push rax".to_string());
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 1".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", 2));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 16".to_string());
                } else {
                    eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable, `{}`", term);
                    exit(1);
                }
            }
            
        }
    }
    asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
    asm = move_to_stack(&"rax".to_string(), &format!("(10 << 8) + ('' << 0)"), asm);
    asm.push("    mov rax, 1".to_string());
    asm.push("    mov rdi, 1".to_string());
    asm.push("    mov rsi, rsp".to_string());
    asm.push(format!("    mov rdx, {}", 2));
    asm.push("    syscall".to_string());
    asm.push("    add rsp, 16".to_string());
    return asm;
}

fn gen_fnc_call(name: &String, args: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if vars.contains_key(name) {
        let f = vars.get(name).unwrap();
        let args = args.concat();
        let args: Vec<&str> = args.split(',').collect();
        let args: Vec<String> = args.iter().map(|&s| s.to_string()).collect();
        if args.len() == f.get(&"args".to_string()).unwrap().get("names").unwrap().len() {
            if args.len() > 0 {
                for i in 0..args.len() {
                    let arg = &args[i];
                    if is_int_lit(arg) {
                        asm = move_to_stack(&"rax".to_string(), arg, asm);
                    } else if vars.get(curr_func).unwrap().get("stack").unwrap().contains_key(arg) {
                        let mut offset = vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().get(arg).unwrap()[0].parse::<i32>().unwrap();
                        offset = (get_stack_height(vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap())-offset)*8;
                        if curr_func != "" {
                            if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(arg) {
                                offset += 8;
                            }
                        }
                        if in_print {
                            offset += 8;
                        }
                        offset += (i as i32)*8;
                        asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {}]", offset), asm);
                    } else if is_fnc_call(arg) {
                        eprintln!("\x1b[1mSyntaxError\x1b[0m: Functions cannot act as arguments to other functions");
                        eprintln!("Try using a variable with the same value instead");
                        exit(1);
                    } else if vars.get("consts").unwrap().get("int").unwrap().contains_key(arg)  {
                        asm = move_to_stack(&"rax".to_string(), &format!("[{}]", arg), asm);
                    }
                    else {
                        eprintln!("\x1b[1mParserError\x1b[0m: Type mismatch");
                        exit(1);
                    }
                }
            }
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Expected {} arguments in function `{}`, but got {}", f.get(&"args".to_string()).unwrap().get("names").unwrap().len(), name, args.len());
            exit(1);
        }
        asm.push(format!("    call {}", name));
        asm.push(format!("    add rsp, {}", args.len()*8));
    } else {
        eprintln!("\x1b[1mParserError\x1b[0m: Used undefined function `{}`", name);
    }
    return asm;
}

fn gen_ret(ret_val: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    if ret_val.len() > 1 
    || has_bin_expr(&ret_val)
    || is_fnc_call(&ret_val[0]) {
        if has_bin_expr(ret_val) {
            asm = gen_bin_expr(ret_val, vars, curr_func, false, asm);
            asm.push("    pop rax".to_string());
        } else if is_fnc_call(&ret_val[0]) {
            let ret_val: Vec<&str> = ret_val[0].split(' ').collect();
            let args: Vec<String> = ret_val[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&ret_val[1].to_string(), &args[0..], vars, curr_func, false, asm);
        }
    } else {
        if is_int_lit(&ret_val[0]) {
            asm.push(format!("    mov rax, {}", ret_val[0]));
        } else if vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&ret_val[0]) 
        || vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&ret_val[0]) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(&ret_val[0]).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&ret_val[0]) {
                    offset += 8;
                }
            }
            asm.push(format!("    mov rax, [rsp + {:?}]", offset))
        } else {
            asm.push(format!("    mov rax, [{}]", ret_val[0]));
        }
    }
    asm.push(format!("    add rsp, {}", 8*vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().len()));
    asm.push("    ret".to_string());

    return asm;
}

fn gen_fnc_dec(name: &String, mut asm: Vec<String>) -> Vec<String> {
    asm.push(format!("{}:", name));
    return asm;
}

fn gen_print(prt: &String, mut asm: Vec<String>) -> Vec<String> {
    let mut buf: Vec<String> = vec![];
    let mut stmt: Vec<String> = vec![];
    let mut pre_stmt: Vec<String> = vec![];
    let mut c = 1;
    for i in 1..prt.len()-1 {
        let byte = i* 8;
        let _char = prt.chars().nth(i).unwrap();
        let piece = format!("('{}' << {})", _char, byte);
        buf.push(piece); 
        if (i % 7 == 0) || i == prt.len()-2 {
            c += 1;
            for j in 1..=buf.len() {
                pre_stmt.push(buf[buf.len()-j].to_string());
            }
            buf.clear();
            if i == prt.len() - 2 {
                if i % 7 != 0 {
                    pre_stmt.insert(0, format!("(10 << {})", byte+8));
                    stmt.insert(0, pre_stmt.join(" + "));
                } else {
                    stmt.insert(0, pre_stmt.join(" + "));
                    stmt.insert(0, format!("(10 << {})", byte+8));
                    c += 1;
                }
            } else {
                stmt.insert(0, pre_stmt.join(" + "));
                pre_stmt.clear();
            }
        }
    }
    stmt.insert(0, "0".to_string());

    for str in stmt {
        asm.push("    mov rax, ".to_string() + &str);
        asm.push("    push rax".to_string());
    }

    asm.push("    mov rax, 1".to_string());
    asm.push("    mov rdi, 1".to_string());
    asm.push("    mov rsi, rsp".to_string());
    asm.push("    mov rdx, ".to_string() + &(prt.len()).to_string());
    asm.push("    syscall".to_string());
    asm.push("    add rsp, ".to_string() + &(c*8).to_string());
    return asm;
}

fn gen_add(lhs: &String, rhs: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if !is_int_lit(rhs) {
        if rhs == "STACK" {
            asm.push("    pop rbx".to_string());
        } else if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(rhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(rhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(rhs) {
                    offset += 8;
                }
            }
            if in_print {
                offset += 8;
            }
            asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
            asm.push("    pop rbx".to_string());
        } else if rhs.contains("fcall") {
            let rhs: Vec<&str> = rhs.split("!").collect();  
            let args: Vec<String> = rhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&rhs[1].to_string(), &args[0..], vars, curr_func, in_print, asm);
            asm.push("mov rbx, rax".to_string());
        } else {
            asm.push(format!("    mov rbx, [{}]", rhs));
        }
    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }
    if !is_int_lit(lhs) {
        if lhs == "STACK" {
            asm.push("    pop rax".to_string());
        } 
        else if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(lhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(lhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(lhs) {
                    offset += 8;
                }
            }
            if in_print {
                offset += 8;
            }
            asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {:?}]", offset), asm);
            asm.push("    pop rax".to_string());
        } else if lhs.contains("fcall") {
            let lhs: Vec<&str> = lhs.split("!").collect();  
            let args: Vec<String> = lhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&lhs[1].to_string(), &args[0..], vars, curr_func, in_print,asm);
        } else {
            asm.push(format!("    mov rax, [{}]", lhs));
        }
    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    add rbx, rax".to_string());
    asm.push("    push rbx".to_string());

    return asm;
}

fn gen_sub(lhs: &String, rhs: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if !is_int_lit(rhs) {
        if rhs == "STACK" {
            asm.push("    pop rbx".to_string());
        } else if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(rhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(rhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(rhs) {
                    offset += 8;
                }
            }
            if in_print {
                offset += 8;
            }
            asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
            asm.push("    pop rbx".to_string());
        } else if rhs.contains("fcall") {
            let rhs: Vec<&str> = rhs.split("!").collect();  
            let args: Vec<String> = rhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&rhs[1].to_string(), &args[0..], vars, curr_func, in_print, asm);
            asm.push("mov rbx, rax".to_string());
        } else {
            asm.push(format!("    mov rbx, [{}]", rhs));
        }
    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }
    if !is_int_lit(lhs) {
        if lhs == "STACK" {
            asm.push("    pop rax".to_string());
        }
        else if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(lhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(lhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(lhs) {
                    offset += 8;
                }
            }
            if in_print {
                offset += 8;
            }
            asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {:?}]", offset), asm);
            asm.push("    pop rax".to_string());
        }  else if lhs.contains("fcall") {
            let lhs: Vec<&str> = lhs.split("!").collect();  
            let args: Vec<String> = lhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&lhs[1].to_string(), &args[0..], vars, curr_func, in_print, asm);
        } else {
            asm.push(format!("    mov rax, [{}]", lhs));
        }
    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    sub rax, rbx".to_string());
    asm.push("    push rax".to_string());

    return asm;
}

fn gen_mul(lhs: &String, rhs: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "0" || !is_int_lit(rhs) {
        if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(rhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(rhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(rhs) {
                    offset += 8;
                }
            }
            if in_print {
                offset += 8;
            }
            asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
            asm.push("    pop rbx".to_string());
        }  else if rhs.contains("fcall") {
            let rhs: Vec<&str> = rhs.split("!").collect();  
            let args: Vec<String> = rhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&rhs[1].to_string(), &args[0..], vars, curr_func, in_print, asm);
            asm.push("mov rbx, rax".to_string());
        } else {
            if rhs == "0" {
                asm.push("    pop rbx".to_string());
            } else {
                asm.push(format!("    mov rbx, [{}]", rhs));
            }
        }

    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }
    if lhs == "0" || !is_int_lit(lhs) {
        if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(lhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(lhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(lhs) {
                    offset += 8;
                }
            }
            if in_print {
                offset += 8;
            }
            asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {:?}]", offset), asm);
            asm.push("    pop rax".to_string());
        }  else if lhs.contains("fcall") {
            let lhs: Vec<&str> = lhs.split("!").collect();  
            let args: Vec<String> = lhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&lhs[1].to_string(), &args[0..], vars, curr_func, in_print, asm);
        } else {
            if lhs == "0" {
                asm.push("    pop rax".to_string());
            } else {
                asm.push(format!("    mov rax, [{}]", lhs));
            }
        }

    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    mul rbx".to_string());
    asm.push("    push rax".to_string());

    return asm;
}

fn gen_div(lhs: &String, rhs: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "0" || !is_int_lit(rhs) {
        if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(rhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(rhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(rhs) {
                    offset += 8;
                }
            }
            if in_print {
                offset += 8;
            }
            asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
            asm.push("    pop rbx".to_string());
        }  else if rhs.contains("fcall") {
            let rhs: Vec<&str> = rhs.split("!").collect();  
            let args: Vec<String> = rhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&rhs[1].to_string(), &args[0..], vars, curr_func, in_print, asm);
            asm.push("mov rbx, rax".to_string());
        } else {
            if rhs == "0" {
                asm.push("    pop rbx".to_string());
            } else {
                asm.push(format!("    mov rbx, [{}]", rhs));
            }
        }

    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }
    if lhs == "0" || !is_int_lit(lhs) {
        if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(lhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(lhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(lhs) {
                    offset += 8;
                }
            }
            if in_print {
                offset += 8;
            }
            asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {:?}]", offset), asm);
            asm.push("    pop rax".to_string());
        }  else if lhs.contains("fcall") {
            let lhs: Vec<&str> = lhs.split("!").collect();  
            let args: Vec<String> = lhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&lhs[1].to_string(), &args[0..], vars, curr_func, in_print, asm);
        } else {
            if lhs == "0" {
                asm.push("    pop rax".to_string());
            } else {
                asm.push(format!("    mov rax, [{}]", lhs));
            }
        }

    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    div rbx".to_string());
    asm.push("    push rax".to_string());

    return asm;
}

fn gen_bin_expr(expr: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    for bin_expr in expr.iter() {
        if bin_expr.contains("PLUS") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_add(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, curr_func, in_print, asm);
        } else if bin_expr.contains("TIMES") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_mul(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, curr_func, in_print, asm);
        } else if bin_expr.contains("MIN") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_sub(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, curr_func, in_print, asm); 
        } else if bin_expr.contains("DIV") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_div(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, curr_func, in_print, asm); 
        }
    }

    return asm;
}

fn gen_const_bin_expr(expr: &[String], mut asm: Vec<String>) -> Vec<String> {
    for bin_expr in expr.iter() {
        let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
        // value checking
        for operand in &bin_expr {
            if !is_int_lit(&operand.to_string())
            || operand != &"PLUS"
            || operand != &"TIMES"
            || operand != &"DIV"
            || operand != &"MIN" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Constant expressions cannot use variables or functions");
                exit(1);
            }
        }
        if bin_expr.contains(&"PLUS") {
            if bin_expr[0] != "0" && bin_expr[2] != "0" {
                asm.push(format!("{} + {}", bin_expr[0], bin_expr[2]));
            } else if bin_expr[0] == "0" && bin_expr[2] != "0" {
                asm.push(format!("+ {}", bin_expr[2]));
            } else if bin_expr[0] != "0" && bin_expr[2] == "0" {
                asm.push(format!("{} + ", bin_expr[0]));
            } else {
                asm.push(format!("+"));
            }
        } else if bin_expr.contains(&"TIMES") {
            if bin_expr[0] != "0" && bin_expr[2] != "0" {
                asm.push(format!("{} * {}", bin_expr[0], bin_expr[2]));
            } else if bin_expr[0] == "0" && bin_expr[2] != "0" {
                asm.push(format!("* {}", bin_expr[2]));
            } else if bin_expr[0] != "0" && bin_expr[2] == "0" {
                asm.push(format!("{} *", bin_expr[0]));
            } else {
                asm.push(format!("*"));
            }
        } else if bin_expr.contains(&"MIN") {
            if bin_expr[0] != "0" && bin_expr[2] != "0" {
                asm.push(format!("{} - {}", bin_expr[0], bin_expr[2]));
            } else if bin_expr[0] == "0" && bin_expr[2] != "0" {
                asm.push(format!("- {}", bin_expr[2]));
            } else if bin_expr[0] != "0" && bin_expr[2] == "0" {
                asm.push(format!("{} -", bin_expr[0]));
            } else {
                asm.push(format!("-"));
            }
        } else if bin_expr.contains(&"DIV") {
            if bin_expr[0] != "0" && bin_expr[2] != "0" {
                asm.push(format!("{} / {}", bin_expr[0], bin_expr[2]));
            } else if bin_expr[0] == "0" && bin_expr[2] != "0" {
                asm.push(format!("/ {}", bin_expr[2]));
            } else if bin_expr[0] != "0" && bin_expr[2] == "0" {
                asm.push(format!("{} /", bin_expr[0]));
            } else {
                asm.push(format!("/"));
            }
        }
    }

    return asm;
}

fn gen_out(out_val: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    if out_val.len() > 1 {
        if has_bin_expr(out_val) {
            asm = gen_bin_expr(out_val, vars, curr_func, false, asm);
        } else {
            exit(1);
        }
    } else {
        if is_bin_expr(&out_val[0]) {
            asm = gen_bin_expr(out_val, vars, curr_func, false, asm);
        } else if is_fnc_call(&out_val[0]) {
            let out_val: Vec<&str> = out_val[0].split(' ').collect();
            let args: Vec<String> = out_val[1..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&out_val[1].to_string(), &args[0..], vars, curr_func, false, asm);
            asm.push("    push rax".to_string());
        } else if is_int_lit(&out_val[0]) {
            asm = move_to_stack(&"rax".to_string(), &out_val[0], asm);
        } else {
            if is_int_lit(&out_val[0]) {
                asm.push(format!("    mov rax, {}", out_val[0]));
            } else if vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&out_val[0]) 
            || vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&out_val[0]){
                let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(&out_val[0]).unwrap()[0].parse::<i32>().unwrap();
                offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
                if curr_func != "" {
                    if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&out_val[0]) {
                        offset += 8;
                    }
                }
                asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {:?}]", offset), asm);
            } else {
                asm.push(format!("    mov rax, [{}]", out_val[0]));
            }
        }
    }
    asm.push("    pop rdi".to_string());
    asm.push("    mov rax, 60".to_string());
    asm.push("    syscall".to_string());

    return asm;
}

fn gen_var_var(var_val: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if var_val.len() > 1 
    || var_val[0].to_string().contains(&"PLUS".to_string()) 
    || var_val[0].to_string().contains(&"TIMES".to_string()) 
    || var_val[0].to_string().contains(&"MIN".to_string())
    || var_val[0].to_string().contains(&"DIV".to_string()) { 
        asm = gen_bin_expr(var_val, vars, curr_func, in_print, asm);
    } else {
        if is_fnc_call(&var_val[0]) {
            let var_val: Vec<&str> = var_val[0].split(' ').collect();
            let args: Vec<String> = var_val[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&var_val[1].to_string(),&args[0..], vars, curr_func, false, asm);
            asm.push("    push rax".to_string());
        } else if vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&var_val[0]) 
        || vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&var_val[0]){
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(&var_val[0]).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&var_val[0]) {
                    offset += 8;
                }
            }
            asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {:?}]", offset), asm);
        } else {
            asm.push("    mov rax, ".to_string() + &var_val[0]);
            asm.push("    push rax".to_string());
        }
    }
    return asm;
}

pub fn write_asm(stmts: Vec<Vec<String>>, name: String, global_start: bool, has_main: bool) {
    let mut vars: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = HashMap::new();
    vars.insert(
        "const".to_string(),
        HashMap::from([
            ("int".to_string(), HashMap::new()),
        ])
    );
    let mut curr_func: String = String::from("");
    let mut stack_height: i32 = 0;
    let asm_name = name.to_owned() + ".asm";
    let mut funcs: Vec<String> = vec![];
    let mut in_func: bool = false;
    let mut data: Vec<String> = vec![];
    let mut start: Vec<String> = vec![];
    let mut text: Vec<String> = vec![];
    let result = File::create(&asm_name);
    let assembly = match result {
        Ok(f) => f,
        Err(_) => {
            eprintln!("\x1b[1mGenerationError\x1b[0m: Could not create writeable asm file");
            exit(1);
        }
    };
    let mut writer: BufWriter<File> = BufWriter::new(assembly);

    for stmt in stmts {
        if &stmt[0] == &"out".to_string() {
            if in_func {
                funcs = gen_out(&stmt[1..],&vars, &curr_func, funcs);
            } else {
                start = gen_out(&stmt[1..], &vars, &curr_func, start);
            }
        } else if &stmt[0] == &"vdec".to_string() {
                if in_func {
                    funcs = gen_var_var(&stmt[4..], &vars, &curr_func, false, funcs);
                } else {
                    start = gen_var_var(&stmt[4..], &vars, &curr_func, false, start);
                }
                if &stmt[1] == &"const".to_string() {
                    data = gen_constant(&stmt[3], &stmt[2], &stmt[4..], data);
                        vars.get_mut(&"const".to_string()).unwrap().get_mut(&stmt[2]).unwrap().insert(
                        stmt[3].clone(),
                        vec![get_constant_len(&stmt[4..].concat()).to_string()]
                    );
                } else {
                    vars.get_mut(&curr_func).unwrap().get_mut("stack").unwrap().insert(
                        stmt[3].clone(), 
                        vec![stack_height.to_string()]
                    );
                    vars.get_mut(&curr_func).unwrap().get_mut("vars").unwrap().get_mut(&stmt[2]).unwrap().push(
                        stmt[3].clone()
                    );
                    vars.get_mut(&curr_func).unwrap().get_mut("vars").unwrap().get_mut("names").unwrap().push(
                        stmt[3].clone()
                    );
                    stack_height += 1;
                }
        } else if &stmt[0] == &"prt".to_string() {
            if in_func {
                funcs = gen_print(&stmt[1], funcs);
            } else {
                start = gen_print(&stmt[1], start);
            }
        } else if &stmt[0] == &"prf".to_string() {
            if in_func {
                funcs = gen_print_f(&stmt[1..], &vars, &curr_func,funcs);
            } else {
                start = gen_print_f(&stmt[1..], &vars, &curr_func,start);
            }
        } else if &stmt[0] == &"pre".to_string() {
            if in_func {
                funcs = gen_print_e(&stmt[1..], &vars, &curr_func,funcs);
            } else {
                start = gen_print_e(&stmt[1..], &vars, &curr_func,start);
            }
        } else if &stmt[0] == &"vass".to_string() {
            if in_func {
                funcs = gen_var_var(&stmt[2..], &vars, &curr_func,false, funcs);
            } else {
                start = gen_var_var(&stmt[2..], &vars, &curr_func,false, start);
            }
            vars.get_mut(&curr_func).unwrap().get_mut("stack").unwrap().insert(
                stmt[2].clone(), 
                vec![stack_height.to_string()]
            );
            stack_height += 1;
        } else if &stmt[0] == "fdec"{
            if !in_func {
                in_func = true;
            }  else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot nest functions");
                exit(1);
            }
            funcs = gen_fnc_dec(&stmt[1], funcs);
            let args = &stmt.clone()[2..stmt.clone().len()].concat();
            let args = args.split(',').collect::<Vec<&str>>();
            let args = args[0..].iter().map(|&s| s.to_string()).collect::<Vec<String>>();
            vars.insert(stmt[1].clone(), HashMap::from([
                ("args".to_string(), HashMap::from([
                    ("int".to_string(), vec![]),
                    ("names".to_string(), vec![]),
                ])),
                ("vars".to_string(), HashMap::from([
                    ("int".to_string(), vec![]),
                    ("names".to_string(), vec![]),
                ])),
                ("stack".to_string(), HashMap::from([
                    ("".to_string(), vec![String::from("-1")]),
                ])),
            ]));
            curr_func = stmt[1].clone();

            for arg in args {
                if arg != "" {
                    let arg = arg.split("COL").collect::<Vec<&str>>();
                    let _type = arg[0];
                    let value = arg[1];
                    vars.get_mut(&stmt[1]).unwrap().get_mut(&"args".to_string()).unwrap().get_mut(&_type.to_string()).unwrap().push(value.to_string());
                    vars.get_mut(&stmt[1]).unwrap().get_mut(&"args".to_string()).unwrap().get_mut(&"names".to_string()).unwrap().push(value.to_string());
                    vars.get_mut(&stmt[1]).unwrap().get_mut(&"stack".to_string()).unwrap().insert(
                        value.to_string(), 
                        vec![stack_height.to_string()]
                    );
                    stack_height += 1;
                }
            }
        } else if &stmt[0] == "endfunc" {
            in_func = false;
            curr_func = "".to_string();
        } else if &stmt[0] == "fcall"{
            if in_func {
                funcs = gen_fnc_call(&stmt[1], &stmt[2..], &vars, &curr_func, false, funcs);
            } else {
                start = gen_fnc_call(&stmt[1], &stmt[2..], &vars, &curr_func, false, start);
            }
            let arg_names = vars.get(&stmt[1]).unwrap().get("args").unwrap().get("names").unwrap().clone();

            for a_name in arg_names{
                if vars.get(&stmt[1]).unwrap().get(&"stack".to_string()).unwrap().contains_key(&a_name) {
                    vars.get_mut(&stmt[1]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&a_name).unwrap().remove(0);
                    vars.get_mut(&stmt[1]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&a_name).unwrap().push(stack_height.to_string());
                    stack_height += 1; 
                } else {
                    vars.get_mut(&stmt[1]).unwrap().get_mut(&"stack".to_string()).unwrap().insert(
                        a_name.to_string(),
                        vec![stack_height.to_string()]
                    );
                    stack_height += 1;
                }
            }
        } else if &stmt[0] == "ret" {
            if in_func {
                funcs = gen_ret(&stmt[1..], &vars, &curr_func,funcs);
            } else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: `ret` can only be used within a function.\nTo exit the program, use `out`");
                exit(1);
            }
        } else {
            exit(11);
        }
    }

    text.push("    global _start".to_string());
    if global_start {
        match writeln!(writer, "section .data") {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
                exit(1);
            },
        }
    }
    for line in data {
        match writeln!(writer, "{}", line) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
                exit(1);
            },
        }
    }
    match writeln!(writer, "section .text") {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
                exit(1);
            },
        }
    for line in text {
        match writeln!(writer, "{}", line) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
                exit(1);
            },
        }
    }

    for line in funcs {
        match writeln!(writer, "{}", line) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
                exit(1);
            },
        }
    }

    for line in start {
        match writeln!(writer, "{}", line) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
                exit(1);
            },
        }
    }
    match writeln!(writer, "_start:") {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
                exit(1);
            },
        }
    if has_main {
        for line in ["    call main", "    mov rdi, rax", 
        "    mov rax, 60", "    syscall"] {
            match writeln!(writer, "{}", line) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
                exit(1);
            },
        }
        }
    }
}
