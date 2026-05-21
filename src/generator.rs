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

fn get_stack_height(vars: &HashMap<String, i32>) -> i32 {
    let mut count: i32 = 0;
    for offset in vars.values() {
        if offset > &count {
            count = *offset;
        }
    }

    return count+1;
}

fn move_to_stack(reg: &String, val: &String, mut asm: Vec<String>) -> Vec<String>  {
    asm.push("    mov ".to_string() + reg + ", " + val);
    asm.push("    push ".to_string() + reg);
    
    return asm;
}

fn gen_print_f(prf: &[String], vars: &HashMap<String, i32>, mut asm: Vec<String>) -> Vec<String> {
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
                asm = gen_bin_expr(&[term.to_owned()], vars, 8, asm);
                asm.push("    pop rax".to_string());
                asm.push(format!("    add rax, 48"));
                asm.push(format!("    shl rax, {}", 8));
                asm.push(format!("    mov rbx, ('' << {})", 0));
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
                asm = gen_fnc_call(&term[1],&term[3..term.len()], asm);
                asm.push(format!("    add rax, 48"));
                asm.push(format!("    shl rax, {}", 8));
                asm.push(format!("    mov rbx, ('' << {})", 0));
                asm.push(format!("    or rax, rbx"));
                asm.push("    push rax".to_string());
                asm.push("    mov rax, 1".to_string());
                asm.push("    mov rdi, 1".to_string());
                asm.push("    mov rsi, rsp".to_string());
                asm.push(format!("    mov rdx, {}", 2));
                asm.push("    syscall".to_string());
                asm.push("    add rsp, 16".to_string());
            } else {
                if let Some(v) = vars.get(term) {
                    let mut offset = *v;
                    offset = (get_stack_height(vars)-offset-1)*8;
                    asm.push(format!("    mov rax, [rsp + {}]", offset+8));
                    asm.push(format!("    add rax, 48"));
                    asm.push(format!("    shl rax, {}", 8));
                    asm.push(format!("    mov rbx, ('' << {})", 0));
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


fn gen_fnc_call(name: &String, args: &[String], mut asm: Vec<String>) -> Vec<String> {
    if !args.contains(&"NONE".to_string()) {
        for i in (6..args.len()).rev() {
            asm = move_to_stack(&"rdi".to_string(), &args[i], asm);
        }
    }

    asm.push(format!("    call {}", name));
    return asm;
}

fn gen_ret(ret_val: &[String], vars: &HashMap<String, i32>, mut asm: Vec<String>, offset: i32) -> Vec<String> {
    if ret_val.len() > 1 
    || has_bin_expr(&ret_val) {
        if has_bin_expr(ret_val) {
            asm = gen_bin_expr(ret_val, vars, asm);
            asm.push("    pop rax".to_string());
        } else {
            exit(11);
        }
    } else {
        if is_int_lit(&ret_val[0]) {
            asm.push(format!("    mov rax, {}", ret_val[0]));
        } else {
            if offset == 1 {
                asm.push(format!("    mov rax, [{}]", ret_val[0]));
            } else {
                asm.push(format!("    mov rax, [rsp + {}]", offset));
            }
        }
    }
    asm.push(format!("    add rsp, {}", get_stack_height(vars)*8));
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

fn gen_add(lhs: &String, rhs: &String, vars: &HashMap<String, i32>, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "0" || !is_int_lit(rhs) {
        if vars.contains_key(rhs) {
            if !is_int_lit(rhs) {
                if let Some(v) = vars.get(rhs) {
                    let mut offset = *v;
                    offset = (get_stack_height(vars)-offset-1)*8;
                    asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
                } else {
                    eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable, `{}`", rhs);
                    exit(1);
                }
            } 
            asm.push("    pop rbx".to_string());
        } else if rhs.contains("fcall") {
            let rhs: Vec<&str> = rhs.split("!").collect();  
            let args: Vec<String> = rhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&rhs[1].to_string(), &args[0..], asm);
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
        if vars.contains_key(lhs) {
            if !is_int_lit(lhs) {
                if let Some(v) = vars.get(lhs) {
                    let mut offset = *v;
                    offset = (get_stack_height(vars)-offset-1)*8;
                    asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
                } else {
                    eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable, `{}`", lhs);
                    exit(1);
                }
            } 
            asm.push("    pop rax".to_string());
        } else if lhs.contains("fcall") {
            let lhs: Vec<&str> = lhs.split("!").collect();  
            let args: Vec<String> = lhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&lhs[1].to_string(), &args[0..], asm);
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
    asm.push("    add rbx, rax".to_string());
    asm.push("    push rbx".to_string());

    return asm;
}

fn gen_sub(lhs: &String, rhs: &String, vars: &HashMap<String, i32>, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "0" || !is_int_lit(rhs) {
        if vars.contains_key(rhs) {
            if !is_int_lit(rhs) {
                if let Some(v) = vars.get(rhs) {
                    let mut offset = *v;
                    offset = (get_stack_height(vars)-offset-1)*8;
                    asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
                } else {
                    eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable, `{}`", rhs);
                    exit(1);
                }
            } 
            asm.push("    pop rbx".to_string());
        } else if rhs.contains("fcall") {
            let rhs: Vec<&str> = rhs.split("!").collect();  
            let args: Vec<String> = rhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&rhs[1].to_string(), &args[0..], asm);
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
        if vars.contains_key(lhs) {
            if !is_int_lit(lhs) {
                if let Some(v) = vars.get(lhs) {
                    let mut offset = *v;
                    offset = (get_stack_height(vars)-offset-1)*8;
                    asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
                } else {
                    eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable, `{}`", rhs);
                    exit(1);
                }
            } 
            asm.push("    pop rax".to_string());
        }  else if lhs.contains("fcall") {
            let lhs: Vec<&str> = lhs.split("!").collect();  
            let args: Vec<String> = lhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&lhs[1].to_string(), &args[0..], asm);
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
    asm.push("    sub rax, rbx".to_string());
    asm.push("    push rax".to_string());

    return asm;
}

fn gen_mul(lhs: &String, rhs: &String, vars: &HashMap<String, i32>, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "0" || !is_int_lit(rhs) {
        if vars.contains_key(rhs) {
            if !is_int_lit(rhs) {
                if let Some(v) = vars.get(rhs) {
                    let mut offset = *v;
                    offset = (get_stack_height(vars)-offset-1)*8;
                    asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
                } else {
                    eprintln!("\x1b[1mCompilationError\x1b[0m: Used undefined or inaccessible variable, `{}`", rhs);
                    exit(1);
                }
            } 
            asm.push("    pop rbx".to_string());
        }  else if rhs.contains("fcall") {
            let rhs: Vec<&str> = rhs.split("!").collect();  
            let args: Vec<String> = rhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&rhs[1].to_string(), &args[0..], asm);
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
        if vars.contains_key(lhs) {
            if !is_int_lit(lhs) {
                if let Some(v) = vars.get(lhs) {
                    let mut offset = *v;
                    offset = (get_stack_height(vars)-offset-1)*8;
                    asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
                } else {
                    eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable, `{}`", rhs);
                    exit(1);
                }
            } 
            asm.push("    pop rax".to_string());
        }  else if lhs.contains("fcall") {
            let lhs: Vec<&str> = lhs.split("!").collect();  
            let args: Vec<String> = lhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&lhs[1].to_string(), &args[0..], asm);
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

fn gen_div(lhs: &String, rhs: &String, vars: &HashMap<String, i32>, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "0" || !is_int_lit(rhs) {
        if vars.contains_key(rhs) {
            if !is_int_lit(rhs) {
                if let Some(v) = vars.get(rhs) {
                    let mut offset = *v;
                    offset = (get_stack_height(vars)-offset-1)*8;
                    asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
                } else {
                    eprintln!("\x1b[1mCompilationError\x1b[0m: Used undefined or inaccessible variable, `{}`", rhs);
                    exit(1);
                }
            } 
            asm.push("    pop rbx".to_string());
        }  else if rhs.contains("fcall") {
            let rhs: Vec<&str> = rhs.split("!").collect();  
            let args: Vec<String> = rhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&rhs[1].to_string(), &args[0..], asm);
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
        if vars.contains_key(lhs) {
            if !is_int_lit(lhs) {
                if let Some(v) = vars.get(lhs) {
                    let mut offset = *v;
                    offset = (get_stack_height(vars)-offset-1)*8;
                    asm = move_to_stack(&"rbx".to_string(), &format!("[rsp + {:?}]", offset), asm);
                } else {
                    eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable, `{}`", rhs);
                    exit(1);
                }
            } 
            asm.push("    pop rax".to_string());
        }  else if lhs.contains("fcall") {
            let lhs: Vec<&str> = lhs.split("!").collect();  
            let args: Vec<String> = lhs[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&lhs[1].to_string(), &args[0..], asm);
            asm.push("mov rbx, rax".to_string());
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

fn gen_bin_expr(expr: &[String], vars: &HashMap<String, i32>,  mut asm: Vec<String>) -> Vec<String> {
    for bin_expr in expr.iter() {
        if bin_expr.contains("PLUS") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_add(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, asm);
        } else if bin_expr.contains("TIMES") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_mul(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, asm);
        } else if bin_expr.contains("MIN") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_sub(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, asm); 
        } else if bin_expr.contains("DIV") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_div(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, asm); 
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

fn gen_out(out_val: &[String], vars: &HashMap<String, i32>, mut asm: Vec<String>, offset: i32) -> Vec<String> {
    if out_val.len() > 1 {
        if has_bin_expr(out_val) {
            asm = gen_bin_expr(out_val, vars, asm);
        } else {
            exit(1);
        }
    } else {
        if is_bin_expr(&out_val[0]) {
            asm = gen_bin_expr(out_val, vars, asm);
        } else if is_fnc_call(&out_val[0]) {
            let out_val: Vec<&str> = out_val[0].split(' ').collect();
            let args: Vec<String> = out_val[1..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&out_val[1].to_string(), &args[0..], asm);
            asm.push("    push rax".to_string());
        } else if is_int_lit(&out_val[0]) {
            asm = move_to_stack(&"rax".to_string(), &out_val[0], asm);
        } else {
            if offset == 1 {
                asm = move_to_stack(&"rax".to_string(), &format!("[{}]", out_val[0]), asm);
            } else {
                asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {}]", offset), asm);
            }
        }
    }
    asm.push("    pop rdi".to_string());
    asm.push("    mov rax, 60".to_string());
    asm.push("    syscall".to_string());

    return asm;
}

fn gen_const_var(const_name: &String, const_val: &[String], mut asm: Vec<String>) -> Vec<String> {
    if const_val.len() > 1 
    || const_val[0].to_string().contains(&"PLUS".to_string()) 
    || const_val[0].to_string().contains(&"TIMES".to_string()) 
    || const_val[0].to_string().contains(&"MIN".to_string())
    || const_val[0].to_string().contains(&"DIV".to_string()) { 

        let _const = gen_const_bin_expr(const_val, vec![]).join(" ");
        asm.push("    ".to_string() + &const_name + " dd " + &_const);
    } else {
        asm.push("    ".to_string() + &const_name + " dd " + &const_val[0]);
    }

    return asm;
}

fn gen_var_var(var_val: &[String], vars: &HashMap<String, i32>, mut asm: Vec<String>) -> Vec<String> {
    if var_val.len() > 1 
    || var_val[0].to_string().contains(&"PLUS".to_string()) 
    || var_val[0].to_string().contains(&"TIMES".to_string()) 
    || var_val[0].to_string().contains(&"MIN".to_string())
    || var_val[0].to_string().contains(&"DIV".to_string()) { 
        asm = gen_bin_expr(var_val, vars, asm);
    } else {
        if is_fnc_call(&var_val[0]) {
            let var_val: Vec<&str> = var_val[0].split(' ').collect();
            let args: Vec<String> = var_val[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&var_val[1].to_string(),&args[0..], asm);
        } else {
            asm.push("    mov rax, ".to_string() + &var_val[0]);
        }
        asm.push("    push rax".to_string());
    }

    return asm;
}

pub fn write_asm(stmts: Vec<Vec<String>>, name: String, global_start: bool, has_main: bool) {
    let mut global_vars: HashMap<String, i32> = HashMap::new();
    let mut local_vars: HashMap<String, i32> = HashMap::new();
    let mut consts: Vec<String> = vec![];
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
    let mut indent: i32 = 0;


    for stmt in stmts {
        if &stmt[0] == &"out".to_string() {
            let mut offset = 0;
            if consts.contains(&stmt[1]) {
                if in_func {
                    funcs = gen_out(&stmt[1..], &local_vars, funcs, 1);
                } else {
                    start = gen_out(&stmt[1..], &global_vars, start, 1);
                }
            } else {
                if is_int_lit(&stmt[1]) {
                    if in_func {
                        funcs = gen_out(&stmt[1..], &local_vars, funcs, offset);
                    } else {
                        start = gen_out(&stmt[1..], &global_vars, start, offset);
                    }
                } else {
                    if has_bin_expr(&stmt[1..]) {
                        if in_func {
                            funcs = gen_out(&stmt[1..], &local_vars, funcs, offset);
                        } else {
                            start = gen_out(&stmt[1..], &global_vars, start, offset);
                        }
                    } else if stmt[1].contains("fcall") {
                        if in_func {
                            funcs = gen_out(&stmt[1..], &local_vars, funcs, offset);
                        } else {
                            start = gen_out(&stmt[1..], &global_vars, start, offset);
                        }
                    } else {
                        if in_func {
                            if let Some(v) = local_vars.get(&stmt[1]) {
                                offset = *v;
                            } else {
                                eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable");
                                exit(1);
                            }
                            offset = (stack_height-offset-1)*8;
                            offset = (stack_height-offset-1)*8;
                            funcs = gen_out(&stmt[1..], &local_vars, funcs, offset);
                        } else {
                            if let Some(v) = global_vars.get(&stmt[1]) {
                                offset = *v;
                            } else {
                                eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable");
                                exit(1);
                            }
                            offset = (stack_height-offset-1)*8;
                            start = gen_out(&stmt[1..], &global_vars, start, offset);
                        }
                    }
                }
            }
        }
        else if &stmt[0] == &"vdec".to_string() {
            if &stmt[1] == &"const".to_string() {
                data = gen_const_var(&stmt[2], &stmt[3..], data);
                consts.push(stmt[2].clone());
            } else {
                if in_func {
                    funcs = gen_var_var(&stmt[3..], &local_vars,funcs);
                    local_vars.insert(stmt[2].clone(), stack_height);
                    stack_height += 1;
                } else {
                    start = gen_var_var(&stmt[3..], &global_vars,start);
                    global_vars.insert(stmt[2].clone(), stack_height);
                    stack_height += 1;
                }
            }
        } else if &stmt[0] == &"prt".to_string() {
            if in_func {
                funcs = gen_print(&stmt[1], funcs);
            } else {
                start = gen_print(&stmt[1], start);
            }
        } else if &stmt[0] == &"vass".to_string() {
            if in_func {
                funcs = gen_var_var(&stmt[2..], &local_vars, funcs);
                *local_vars.get_mut(&stmt[1]).unwrap() = stack_height;
            } else {
                start = gen_var_var(&stmt[2..], &global_vars, start);
                *global_vars.get_mut(&stmt[1]).unwrap() = stack_height;
            }
            stack_height += 1;
        } else if &stmt[0] == "fdec"{
            if !in_func {
                in_func = true;
            }
            funcs = gen_fnc_dec(&stmt[1], funcs);
            indent += 1;
        } else if &stmt[0] == "endfunc" {
            indent -= 1;
            if indent == 0 {
                in_func = false;
            }
        } else if &stmt[0] == "fcall"{
            if in_func {
                funcs.push(format!("    call {}", &stmt[1]));
            } else {
                start.push(format!("    call {}", &stmt[1]));
            }
        } else if &stmt[0] == "ret" {
            if in_func {
                let mut offset = 0;
                if consts.contains(&stmt[1]) {
                    funcs = gen_ret(&stmt[1..], &local_vars, funcs, 1);
                } else {
                    if is_int_lit(&stmt[1]) {
                        funcs = gen_ret(&stmt[1..], &local_vars, funcs, offset);
                    } else {
                        if has_bin_expr(&stmt[1..]) {
                                funcs = gen_ret(&stmt[1..], &local_vars, funcs, offset);
                        } else if stmt[1].contains("fcall") {
                            funcs = gen_ret(&stmt[1..], &local_vars, funcs, offset);
                        } else {
                            if let Some(v) = local_vars.get(&stmt[1]) {
                                offset = *v;
                            }  else {
                                eprintln!("\x1b[1mParserError\x1b[0m: Used undefined or inaccessible variable");
                                exit(1);
                            }
                            offset = (stack_height-offset-1)*8;
                            funcs = gen_ret(&stmt[1..], &local_vars, funcs, offset);
                        }
                    }
                }
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
