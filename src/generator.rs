

use std::{collections::{HashMap}, fs::File, io::{BufWriter, Write}, process::exit};



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
    if expr.contains(&"+".to_string()) 
    || expr.contains(&"*".to_string()) 
    || expr.contains(&"-".to_string())
    || expr.contains(&"/".to_string()) {
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

fn get_external_functions(extern_scopes: &Vec<HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>>) -> Vec<String> {
    let mut extern_functions: Vec<String> = vec![];
    for scope in extern_scopes {
        for function in scope.keys().filter(|key|!is_int_lit(key)) {
            extern_functions.push(function.clone());
        }
    }
    return extern_functions;
}

fn get_var_size(stack: &HashMap<String, Vec<String>>, var: &String) -> i32 {
    return stack.get(var).unwrap()[1].parse::<i32>().unwrap() + 1;
}

fn get_constant_len(constant: &String) -> i32 {
    let mut size: i32 = 0;
    for _ch in constant.chars() {
        size += 1;
    }
    return size;
}

fn get_stack_height(stack: &HashMap<String, Vec<String>>, mode: &String) -> i32 {
    let mut count: i32 = -1;
    for var in stack.keys() {
        let off = stack.get(var).unwrap();
        if off.len() <= 2 {
            continue;
        }
        if mode == "args" {
            if off[2] != "arg" {
                continue;
            }
        } else if mode == "vars"{
            if off[2] != "var" {
                continue;
            }
        }
        if off[0].parse::<i32>().unwrap() + off[1].parse::<i32>().unwrap() > count { 
            count = off[0].parse::<i32>().unwrap() + off[1].parse::<i32>().unwrap();
        }
    }   
    return count;
}

fn move_to_stack(reg: &String, val: &String, mut asm: Vec<String>) -> Vec<String>  {
    asm.push("    mov ".to_string() + reg + ", " + val);
    asm.push("    push ".to_string() + reg);
    
    return asm;
}


fn gen_require(external_functions: &Vec<String>, mut asm: Vec<String>) -> Vec<String> {
    for function in external_functions {
        asm.push(format!("    extern {function}"));
    }
    return asm;
}

fn gen_if(jumps: &i32, section: &Vec<i32>, condition: &String, lhs: &[String], rhs: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    if has_bin_expr(lhs) {
        asm = gen_bin_expr(lhs, vars, curr_func, false, asm);
        asm.push(format!("    pop rbx"));
    } else if is_fnc_call(&lhs[0]) {
        asm = gen_fnc_call(&lhs[1], &lhs[2..], vars, curr_func, false, asm);
        asm.push(format!("    mov rbx, rax"));
    } else if vars.1.get(curr_func).unwrap().get("vars").unwrap().get("all").unwrap().contains(&lhs[0]) 
    || vars.1.get(curr_func).unwrap().get("args").unwrap().get("all").unwrap().contains(&lhs[0]){
        let mut offset = vars.1.get(curr_func).unwrap().get("stack").unwrap().get(&lhs[0]).unwrap()[0].parse::<i32>().unwrap();
        offset = (get_stack_height(vars.1.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
        if curr_func != "" {
            if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&lhs[0]) {
                offset += 8;
            }
        }
        asm.push(format!("    mov rbx, [rsp + {:?}]", offset));
    } else if vars.get("const").unwrap().get("int").unwrap().contains_key(&lhs[0]) 
    || vars.get("const").unwrap().get("bool").unwrap().contains_key(&lhs[0]) {
        asm.push(format!("    mov rbx, [{}]", lhs[0]));
    } else {
        if is_int_lit(&lhs[0]) {
            asm.push(format!("    mov rbx, {}", lhs[0]));
        } else {
            if &lhs[0] == "True" {
                asm.push(format!("    mov rbx, 1"));
            } else if &lhs[0] == "False" {
                asm.push(format!("    mov rbx, 0"));
            }
        }
    }

    if has_bin_expr(rhs) {
        asm = gen_bin_expr(rhs, vars, curr_func, false, asm);
        asm.push(format!("    pop rax"));
    } else if is_fnc_call(&rhs[0]) {
        asm = gen_fnc_call(&rhs[1], &rhs[2..], vars, curr_func, false, asm);
    } else if vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&rhs[0]) 
    || vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&rhs[0]){
        let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(&rhs[0]).unwrap()[0].parse::<i32>().unwrap();
        offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
        if curr_func != "" {
            if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&rhs[0]) {
                offset += 8;
            }
        }
        asm.push(format!("    mov rax, [rsp + {:?}]", offset));
    } else if vars.get("const").unwrap().get("int").unwrap().contains_key(&rhs[0]) 
    || vars.get("const").unwrap().get("bool").unwrap().contains_key(&rhs[0]) {
        asm.push(format!("    mov rax, [{}]", rhs[0]));
    } else {
        if is_int_lit(&rhs[0]) {
            asm.push(format!("    mov rax, {}", rhs[0]));
        } else {
            if &rhs[0] == "True" {
                asm.push(format!("    mov rax, 1"));
            } else if &rhs[0] == "False" {
                asm.push(format!("    mov rax, 0"));
            }
        }
    } 

    asm.push(format!("    cmp rbx, rax")); 
    if section.len() > 0 {
        match condition.as_str() {
            "=" => asm.push(format!("    je .section{}",  [section[0..section.len()-1].iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string(), "1".to_string()].join("_"))), 
            "!" => asm.push(format!("    jne .section{}", [section[0..section.len()-1].iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string(), "1".to_string()].join("_"))),
            ">" => asm.push(format!("    jg .section{}",  [section[0..section.len()-1].iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string(), "1".to_string()].join("_"))),
            "<" => asm.push(format!("    jl .section{}",  [section[0..section.len()-1].iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string(), "1".to_string()].join("_"))),
            ">=" => asm.push(format!("   jge .section{}", [section[0..section.len()-1].iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string(), "1".to_string()].join("_"))),
            "<=" => asm.push(format!("   jle .section{}", [section[0..section.len()-1].iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string(), "1".to_string()].join("_"))),
            _ => exit(1),
        }
    } else {
        match condition.as_str() {
            "=" => asm.push(format!("    je .section_1")), 
            "!" => asm.push(format!("    jne .section_1")), 
            ">" => asm.push(format!("    jg .section_1")),  
            "<" => asm.push(format!("    jl .section_1")),  
            ">=" => asm.push(format!("   jge .section_1")),
            "<=" => asm.push(format!("   jle .section_1")),
            _ => exit(1),
        }
    }
    if section.len() == 0 {
        if jumps >= &1 { asm.push(format!("    jmp .section_2"));}
        asm.push(format!(".section_1:"));
    } else if section.len() == 1 {
        if jumps >= &1 {asm.push(format!("    jmp .section_{}", section[0]+1));}
        asm.push(format!(".section_{}_{}:", section[0]+1, "1"))
    } else {
        if jumps >= &1 {
            asm.push(format!("    jmp .section_{}", [section[0..section.len()-2].iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+1).to_string()].join("_")));
        }
        asm.push(format!(".section_{}:", [section[0..section.len()-1].iter().map(|i| i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
    }
    return asm;
}

fn gen_constant(name : &String, _type: &String, value: &[String], mut asm: Vec<String>) -> Vec<String> {    
    if _type == "int" {
        if has_bin_expr(value) {
            asm = gen_const_bin_expr(value, asm);
        } else {
            asm.push(format!("    {name} dq {}", value.concat()));
        }
    } else if _type == "str" {
        asm.push(format!("    {name} db {}", value.concat()));
    } else if _type == "bool" {
        asm.push(format!("    {name} dw {}", value.concat()));
    } else {
        println!("unknown constant type");
        exit(11);
    }
    return asm;
}

fn gen_print_e(prf: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    let mut stmt: Vec<String> = vec![];
    let rsp_offset = 16;
    for term in prf {
        if term.chars().nth(0) == Some('"') && !vars.get("const").unwrap().get("str").unwrap().contains_key(term){
            asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
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
                    asm.push(format!("    add rsp, {}", rsp_offset));
                }
            }
        } else {
            if is_bin_expr(term) {
                asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
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
                asm.push(format!("    add rsp, {}", rsp_offset));
            } else if is_fnc_call(term){
                let term: Vec<&str> = term.split_ascii_whitespace().collect();
                let term: Vec<String> = term.iter().map(|&a| a.to_string()).collect();
                if vars.get("funcs").unwrap().get("str").unwrap().contains_key(&term[1]) {
                    asm = gen_fnc_call(&term[1],&term[2..], vars, curr_func, true,asm);
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 2".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", vars.get("funcs").unwrap().get("str").unwrap().get(&term[1]).unwrap()[0]));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 512".to_string());
                } else {
                    asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
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
                    asm.push(format!("    add rsp, {}", rsp_offset));
                }
                
            } else {
                asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
                if vars.get(curr_func).unwrap().get("stack").unwrap().contains_key(term)
                {
                    let mut offset = vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().get(term).unwrap()[0].parse::<i32>().unwrap();
                    offset = (get_stack_height(vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap(), &"all".to_string())-offset)*8;
                    if curr_func != "" {
                        if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(term) {
                            offset += 8;
                        }
                    }
                    for i in 0..get_var_size(vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap(), term) {
                        if i > 0 {
                            asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
                        }
                        asm.push(format!("    mov rax, [rsp + {}]", offset+8-(i*8)));
                        if vars.get(curr_func).unwrap().get("args").unwrap().get("int").unwrap().contains(term) 
                        || vars.get(curr_func).unwrap().get("vars").unwrap().get("int").unwrap().contains(term) {
                            asm.push(format!("    add rax, 48"));
                        }
                        asm.push(format!("    mov rbx, ('' << {})", 8));
                        asm.push(format!("    or rax, rbx"));
                        asm.push("    push rax".to_string());
                        asm.push("    mov rax, 1".to_string());
                        asm.push("    mov rdi, 2".to_string());
                        asm.push("    mov rsi, rsp".to_string());
                        asm.push(format!("    mov rdx, {}", 2));
                        asm.push("    syscall".to_string());
                        asm.push(format!("    add rsp, {}", rsp_offset));
                    }                
                } else if vars.get("const").unwrap().get("int").unwrap().contains_key(term)  {
                    asm.push(format!("    mov rax, {}", term));
                    asm.push(format!("    add rax, 48"));
                    asm.push(format!("    mov rbx, ('' << {})", 8));
                    asm.push(format!("    or rax, rbx"));
                    asm.push("    push rax".to_string());
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 2".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", 2));
                    asm.push("    syscall".to_string());
                    asm.push(format!("    add rsp, {}", rsp_offset));
                } else if vars.get("const").unwrap().get("str").unwrap().contains_key(term) {
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 2".to_string());
                    asm.push(format!("    mov rsi, {}", term));
                    asm.push(format!("    mov rdx, {}", vars.get("const").unwrap().get("str").unwrap().get(term).unwrap()[0].parse::<i32>().unwrap() -2));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 8".to_string());
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
    asm.push(format!("    add rsp, {}", rsp_offset));
    return asm;
}

fn gen_print_f(prf: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    let mut stmt: Vec<String> = vec![];
    let rsp_offset: i32 = 16;
    for term in prf {
        if term.chars().nth(0) == Some('"') && !vars.get("const").unwrap().get("str").unwrap().contains_key(term){
            asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
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
                    asm.push(format!("    add rsp, {}", rsp_offset));
                }
            }
        } else {
            if is_bin_expr(term) {
                asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
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
                asm.push(format!("    add rsp, {}", rsp_offset));
            } else if is_fnc_call(term) {
                let term: Vec<&str> = term.split_ascii_whitespace().collect();
                let term: Vec<String> = term.iter().map(|&a| a.to_string()).collect();
                if vars.get("funcs").unwrap().get("str").unwrap().contains_key(&term[1]) {
                    asm = gen_fnc_call(&term[1],&term[2..], vars, curr_func, true,asm);
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 1".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", vars.get("funcs").unwrap().get("str").unwrap().get(&term[1]).unwrap()[0]));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 512".to_string());
                } else {
                    asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
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
                    asm.push(format!("    add rsp, {}", rsp_offset));
                }
            } else {
                asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
                if vars.get(curr_func).unwrap().get("stack").unwrap().contains_key(term)
                {
                    let mut offset = vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().get(term).unwrap()[0].parse::<i32>().unwrap();
                    offset = (get_stack_height(vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap(), &"all".to_string())-offset)*8;
                    if curr_func != "" {
                        if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(term) {
                            offset += 8;
                        }
                    }
                    for i in 0..get_var_size(vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap(), term) {
                        if i > 0 {
                            asm = move_to_stack(&"rax".to_string(), &"0".to_string(), asm);
                        }
                        asm.push(format!("    mov rax, [rsp + {}]", offset+8-(i*8)));
                        if vars.get(curr_func).unwrap().get("args").unwrap().get("int").unwrap().contains(term) 
                        || vars.get(curr_func).unwrap().get("vars").unwrap().get("int").unwrap().contains(term) {
                            asm.push(format!("    add rax, 48"));
                        }
                        asm.push(format!("    mov rbx, ('' << {})", 8));
                        asm.push(format!("    or rax, rbx"));
                        asm.push("    push rax".to_string());
                        asm.push("    mov rax, 1".to_string());
                        asm.push("    mov rdi, 1".to_string());
                        asm.push("    mov rsi, rsp".to_string());
                        asm.push(format!("    mov rdx, {}", 2));
                        asm.push("    syscall".to_string());
                        asm.push(format!("    add rsp, {}", rsp_offset));
                    }                
                } else if vars.get("const").unwrap().get("int").unwrap().contains_key(term)  {
                    asm.push(format!("    mov rax, {}", term));
                    asm.push(format!("    add rax, 48"));
                    asm.push(format!("    mov rbx, ('' << {})", 8));
                    asm.push(format!("    or rax, rbx"));
                    asm.push("    push rax".to_string());
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 1".to_string());
                    asm.push("    mov rsi, rsp".to_string());
                    asm.push(format!("    mov rdx, {}", 2));
                    asm.push("    syscall".to_string());
                    asm.push(format!("    add rsp, {}", rsp_offset));
                } else if vars.get("const").unwrap().get("str").unwrap().contains_key(term) {
                    asm.push("    mov rax, 1".to_string());
                    asm.push("    mov rdi, 1".to_string());
                    asm.push(format!("    mov rsi, {}", term));
                    asm.push(format!("    mov rdx, {}", vars.get("const").unwrap().get("str").unwrap().get(term).unwrap()[0].parse::<i32>().unwrap() -2));
                    asm.push("    syscall".to_string());
                    asm.push("    add rsp, 8".to_string());
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
    asm.push(format!("    add rsp, {}", rsp_offset));
    return asm;
}

fn gen_fnc_call(name: &String, args: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if vars.contains_key(name) {
        let f = vars.get(name).unwrap();
        let args = args.to_vec();
        if args.len() == f.get(&"args".to_string()).unwrap().get("names").unwrap().len() {
            if vars.get("funcs").unwrap().get("str").unwrap().contains_key(name) {
                asm.push("    sub rsp, 512".to_string());
                asm.push("    lea rdi, [rsp]".to_string());
            }
            if args.len() > 0 {
                for i in 0..args.len() {
                    let arg = &args[i];
                    if is_int_lit(arg) {
                        asm = move_to_stack(&"rax".to_string(), arg, asm);
                    } else if vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&arg)
                    || vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&arg) {
                        let mut offset = vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().get(arg).unwrap()[0].parse::<i32>().unwrap();
                        offset = (get_stack_height(vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap(), &"all".to_string())-offset)*8;
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
                    } else if vars.get("const").unwrap().get("int").unwrap().contains_key(arg) 
                    || vars.get("const").unwrap().get("str").unwrap().contains_key(arg) 
                    || vars.get("const").unwrap().get("bool").unwrap().contains_key(arg){
                        asm = move_to_stack(&"rax".to_string(), &format!("{}", arg), asm);
                    } else if matches!(arg.as_str(), "True" | "False") {
                        match arg.as_str() {
                            "True" => asm = move_to_stack(&"rax".to_string(), &format!("1"), asm),
                            _ => asm = move_to_stack(&"rax".to_string(), &format!("0"), asm),
                        }
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
        if vars.get("funcs").unwrap().get("str").unwrap().contains_key(name) && !in_print {
            asm.push(format!("    add rsp, 512"));
        }
        asm.push(format!("    add rsp, {}", args.len()*8));
    } else {
        eprintln!("\x1b[1mParserError\x1b[0m: Used undefined function `{}`", name);
    }
    return asm;
}

fn gen_ret(ret_val: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    if vars.get("funcs").unwrap().get("int").unwrap().contains_key(curr_func) {
        if has_bin_expr(&ret_val) {
            asm = gen_bin_expr(ret_val, vars, curr_func, false, asm);
            asm.push("    pop rax".to_string());
        } else if is_fnc_call(&ret_val[0]) {
            let ret_val: Vec<&str> = ret_val[0].split(' ').collect();
            let args: Vec<String> = ret_val[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&ret_val[1].to_string(), &args[0..], vars, curr_func, false, asm);
        } else if vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&ret_val[0]) 
        || vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&ret_val[0]) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(&ret_val[0]).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&ret_val[0]) {
                    offset += 8;
                }
            }
            asm.push(format!("    mov rax, [rsp + {:?}]", offset))
        } else {
            asm.push(format!("    mov rax, {}", ret_val[0]));
        }
    } else if vars.get("funcs").unwrap().get("str").unwrap().contains_key(curr_func) {
        if vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&ret_val[0]) 
        || vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&ret_val[0]) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(&ret_val[0]).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&ret_val[0]) {
                    offset += 8;
                }
            }
            for i in 0..get_var_size(vars.get(curr_func).unwrap().get("stack").unwrap(), &ret_val[0]) {
                asm.push(format!("    mov al, [rsp+{}]", offset - i*8));
                asm.push(format!("    mov [rdi+{}], al", i));
            }
        } else {
            for i in 0..ret_val.len() {
                asm.push(format!("    mov byte [rdi+{}], '{}'", i, ret_val[i]));
            }
            asm.push(format!("    mov byte [rdi+{}], 0", ret_val.len()));
        }
    } else if vars.get("funcs").unwrap().get("bool").unwrap().contains_key(curr_func) {
        if is_fnc_call(&ret_val[0]) {
            let ret_val: Vec<&str> = ret_val[0].split(' ').collect();
            let args: Vec<String> = ret_val[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&ret_val[1].to_string(), &args[0..], vars, curr_func, false, asm);
        } else if vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&ret_val[0]) 
        || vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&ret_val[0]) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(&ret_val[0]).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
            if curr_func != "" {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&ret_val[0]) {
                    offset += 8;
                }
            }
            asm.push(format!("    mov rax, [rsp + {:?}]", offset))
        } else {
            match ret_val[0].as_str() {
                "True" => asm.push(format!("    mov rax, 1")),
                _ => asm.push(format!("    mov rax, 0")),
            }
        }
    }
    asm.push(format!("    mov rsp, rbp"));
    asm.push(format!("    pop rbp"));
    //asm.push(format!("    add rsp, {}", 8*(1+get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"vars".to_string()))));
    asm.push("    ret".to_string());

    return asm;
}

fn gen_fnc_dec(name: &String, mut asm: Vec<String>) -> Vec<String> {
    asm.push(format!("{}:", name));
    asm.push(format!("    push rbp"));
    asm.push(format!("    mov rbp, rsp"));
    asm.push(format!("    sub rsp, 4096"));
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

fn gen_add(lhs: &String, rhs: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if !is_int_lit(rhs) {
        if rhs == "STACK" {
            asm.push("    pop rbx".to_string());
        } else if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(rhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(rhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
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
            asm.push(format!("    mov rbx, {}", rhs));
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
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
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
            asm.push(format!("    mov rax, {}", lhs));
        }
    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    add rbx, rax".to_string());
    asm.push("    push rbx".to_string());

    return asm;
}

fn gen_sub(lhs: &String, rhs: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if !is_int_lit(rhs) {
        if rhs == "STACK" {
            asm.push("    pop rbx".to_string());
        } else if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(rhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(rhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
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
            asm.push(format!("    mov rbx, {}", rhs));
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
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
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
            asm.push(format!("    mov rax, {}", lhs));
        }
    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    sub rax, rbx".to_string());
    asm.push("    push rax".to_string());

    return asm;
}

fn gen_mul(lhs: &String, rhs: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "0" || !is_int_lit(rhs) {
        if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(rhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(rhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
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
            if rhs == "STACK" {
                asm.push("    pop rbx".to_string());
            } else {
                asm.push(format!("    mov rbx, {}", rhs));
            }
        }

    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }
    if lhs == "STACK" || !is_int_lit(lhs) {
        if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(lhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(lhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
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
            if lhs == "STACK" {
                asm.push("    pop rax".to_string());
            } else {
                asm.push(format!("    mov rax, {}", lhs));
            }
        }

    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    mul rbx".to_string());
    asm.push("    push rax".to_string());

    return asm;
}

fn gen_div(lhs: &String, rhs: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "0" || !is_int_lit(rhs) {
        if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(rhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(rhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
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
                asm.push(format!("    mov rbx, {}", rhs));
            }
        }

    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }
    if lhs == "0" || !is_int_lit(lhs) {
        if vars.get(curr_func).unwrap().get(&"stack".to_string()).unwrap().contains_key(lhs) {
            let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(lhs).unwrap()[0].parse::<i32>().unwrap();
            offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
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
                asm.push(format!("    mov rax, {}", lhs));
            }
        }

    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    div rbx".to_string());
    asm.push("    push rax".to_string());

    return asm;
}

fn gen_bin_expr(expr: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    for bin_expr in expr.iter() {
        if bin_expr.contains("+") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_add(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, curr_func, in_print, asm);
        } else if bin_expr.contains("*") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_mul(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, curr_func, in_print, asm);
        } else if bin_expr.contains("-") {
            let bin_expr: Vec<&str> = bin_expr.split(' ').collect();
            asm = gen_sub(&bin_expr[0].to_string(), &bin_expr[2].to_string(), vars, curr_func, in_print, asm); 
        } else if bin_expr.contains("/") {
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
            || operand != &"+"
            || operand != &"*"
            || operand != &"/"
            || operand != &"-" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Constant expressions cannot use variables or functions");
                exit(1);
            }
        }
        if bin_expr.contains(&"+") {
            if bin_expr[0] != "0" && bin_expr[2] != "0" {
                asm.push(format!("{} + {}", bin_expr[0], bin_expr[2]));
            } else if bin_expr[0] == "0" && bin_expr[2] != "0" {
                asm.push(format!("+ {}", bin_expr[2]));
            } else if bin_expr[0] != "0" && bin_expr[2] == "0" {
                asm.push(format!("{} + ", bin_expr[0]));
            } else {
                asm.push(format!("+"));
            }
        } else if bin_expr.contains(&"*") {
            if bin_expr[0] != "0" && bin_expr[2] != "0" {
                asm.push(format!("{} * {}", bin_expr[0], bin_expr[2]));
            } else if bin_expr[0] == "0" && bin_expr[2] != "0" {
                asm.push(format!("* {}", bin_expr[2]));
            } else if bin_expr[0] != "0" && bin_expr[2] == "0" {
                asm.push(format!("{} *", bin_expr[0]));
            } else {
                asm.push(format!("*"));
            }
        } else if bin_expr.contains(&"-") {
            if bin_expr[0] != "0" && bin_expr[2] != "0" {
                asm.push(format!("{} - {}", bin_expr[0], bin_expr[2]));
            } else if bin_expr[0] == "0" && bin_expr[2] != "0" {
                asm.push(format!("- {}", bin_expr[2]));
            } else if bin_expr[0] != "0" && bin_expr[2] == "0" {
                asm.push(format!("{} -", bin_expr[0]));
            } else {
                asm.push(format!("-"));
            }
        } else if bin_expr.contains(&"/") {
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

fn gen_out(out_val: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    if out_val.len() > 1 {
        if has_bin_expr(out_val) {
            asm = gen_bin_expr(out_val, vars, curr_func, false, asm);
        } else {
            println!("improper out size?");
            exit(11);
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
                offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
                if curr_func != "" {
                    if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&out_val[0]) {
                        offset += 8;
                    }
                }
                asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {:?}]", offset), asm);
            } else {
                asm.push(format!("    mov rax, {}", out_val[0]));
            }
        }
    }
    asm.push("    pop rdi".to_string());
    asm.push("    mov rax, 60".to_string());
    asm.push("    syscall".to_string());

    return asm;
}

fn gen_var_var(var_val: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if has_bin_expr(var_val) { 
        asm = gen_bin_expr(var_val, vars, curr_func, in_print, asm);
    } else  if is_fnc_call(&var_val[0]) {
        let var_val: Vec<&str> = var_val[0].split(' ').collect();
        let args: Vec<String> = var_val[2..].iter().map(|s| s.to_string()).collect();
        asm = gen_fnc_call(&var_val[1].to_string(),&args[0..], vars, curr_func, false, asm);
        asm.push("    push rax".to_string());
    } else if vars.get(curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&var_val[0])
    || vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&var_val[0]) {
        let mut offset = vars.get(curr_func).unwrap().get("stack").unwrap().get(&var_val[0]).unwrap()[0].parse::<i32>().unwrap();
        offset = (get_stack_height(vars.get(curr_func).unwrap().get("stack").unwrap(), &"all".to_string())-offset)*8;
        if curr_func != "" {
            if vars.get(curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&var_val[0]) {
                offset += 8;
            }
        }
        if get_var_size(vars.get(curr_func).unwrap().get("stack").unwrap(), &var_val[0]) == 1 {
            asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {:?}]", offset), asm);
        } else {
            for i in 0..get_var_size(vars.get(curr_func).unwrap().get("stack").unwrap(), &var_val[0]) {
                asm = move_to_stack(&"rax".to_string(), &format!("[rsp + {:?}]", offset-(i*8)), asm);
            }
        }
    } else if var_val == &["True"] {
        asm.push("    mov rax, 1".to_string());
        asm.push("    push rax".to_string());
    } else if var_val == &["False"] {
        asm.push("    mov rax, 0".to_string());
        asm.push("    push rax".to_string()); 
    } else {
        if var_val.len() == 1 {
            if is_int_lit(&var_val[0]) {
                asm.push("    mov rax, ".to_string() + &var_val[0]);
                asm.push("    push rax".to_string());
            } else {
                asm.push("    mov al, ".to_string() + &format!("'{}'", var_val[0].chars().nth(0).unwrap()));
                asm.push("    push rax".to_string());
            }
        } else {
            for word in var_val {
                for i in (0..word.len()).rev() {
                    let ch: char = word.chars().nth(i).unwrap();
                    asm.push("    mov al, ".to_string() + &format!("'{ch}'"));
                    asm.push("    push rax".to_string());
                }
            } 
        }
    }
    return asm;
}

pub fn write_asm(stmts: Vec<Vec<String>>, name: String, build_dir: String, extern_scopes: &Vec<HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>>) {
    let mut constants: HashMap<String, HashMap<String, Vec<String>>> = HashMap::from([
        ("int".to_string(), HashMap::from([("names".to_string(), vec![]), ("lengths".to_string(), vec![]),])),
        ("str".to_string(), HashMap::from([("names".to_string(), vec![]), ("lengths".to_string(), vec![]),])),
        ("bool".to_string(), HashMap::from([("names".to_string(), vec![]), ("lengths".to_string(), vec![]),])),
        ("all".to_string(), HashMap::from([("names".to_string(), vec![]), ("lengths".to_string(), vec![]),])),
    ]);
    let mut functions: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = HashMap::new();
    for scope in extern_scopes {
        functions.iter().chain(scope);
    }
    let mut if_statements: Vec<(String, i32, bool)> = vec![];
    let mut if_depth: u32 = 0;
    let mut section: Vec<i32> = vec![];
    let mut is_module: bool = false;
    let mut curr_func: String = String::from("");
    let mut stack_height: i32 = 0;
    let asm_name = build_dir.clone() + &name.to_owned() + ".asm";
    let mut funcs: Vec<String> = vec![];
    let mut in_func: bool = false;
    let mut data: Vec<String> = vec![];
    let mut start: Vec<String> = vec![];
    let mut text: Vec<String> = vec![];
    let result = File::create(&asm_name);
    let assembly = match result {
        Ok(f) => {f},
        Err(_) => {
            eprintln!("\x1b[1mGenerationError\x1b[0m: Could not create writeable asm file");
            exit(1);
        }
    };
    let mut writer: BufWriter<File> = BufWriter::new(assembly);
    let mut i = 0;
    while i < stmts.len() {
        let stmt = &stmts[i];
        if &stmt[0] == &"out".to_string() {
            if in_func{
                funcs = gen_out(&stmt[1..],(&constants, &functions), &curr_func, funcs);
            } else  {
                start = gen_out(&stmt[1..], (&constants, &functions), &curr_func, start);
            }
        } else if &stmt[0] == &"vdec".to_string() {
                if &stmt[1] == &"const".to_string() {
                    data = gen_constant(&stmt[3], &stmt[2], &stmt[4..], data);
                    constants.get_mut(&stmt[2]).unwrap().get_mut(&"names".to_string()).unwrap().push(stmt[3].clone());
                    constants.get_mut(&stmt[2]).unwrap().get_mut(&"lengths".to_string()).unwrap().push(get_constant_len(&stmt[4..].concat()).to_string());
                } else {
                    if in_func {
                        funcs = gen_var_var(&stmt[4..], (&constants, &functions), &curr_func, false, funcs);
                    } else {
                        start = gen_var_var(&stmt[4..], (&constants, &functions), &curr_func, false, start);
                    }
                    functions.get_mut(&curr_func).unwrap().get_mut(&"vars".to_string()).unwrap().get_mut(&stmt[2]).unwrap().push(stmt[3].clone());
                    functions.get_mut(&curr_func).unwrap().get_mut(&"vars".to_string()).unwrap().get_mut(&"all".to_string()).unwrap().push(stmt[3].clone());
                    functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"stack height".to_string()).unwrap().push(stack_height.to_string());
                    functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"variable name".to_string()).unwrap().push(stmt[3].clone().to_owned());
                    functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"variable category".to_string()).unwrap().push("var".to_string());
                    functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"spillover".to_string()).unwrap().push((stmt.len()-5).to_string());
                    stack_height += stmt.len() as i32 -4;
                }
        } else if &stmt[0] == &"prt".to_string() {
            if in_func {
                funcs = gen_print(&stmt[1], funcs);
            } else {
                start = gen_print(&stmt[1], start);
            }
        } else if &stmt[0] == &"prf".to_string() {
            if in_func {
                funcs = gen_print_f(&stmt[1..], (&constants, &functions), &curr_func, funcs);
            } else {
                start = gen_print_f(&stmt[1..], (&constants, &functions), &curr_func, start);
            }
        } else if &stmt[0] == &"pre".to_string() {
            if in_func {
                funcs = gen_print_e(&stmt[1..], (&constants, &functions), &curr_func, funcs);
            } else {
                start = gen_print_e(&stmt[1..], (&constants, &functions), &curr_func, start);
            }
        } else if &stmt[0] == &"vass".to_string() {
            if in_func {
                funcs = gen_var_var(&stmt[2..], (&constants, &functions), &curr_func,false, funcs);
            } else {
                start = gen_var_var(&stmt[2..], (&constants, &functions), &curr_func,false, start);
            }
            functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"stack height".to_string()).unwrap().push(stack_height.to_string());
            functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"variable name".to_string()).unwrap().push(stmt[3].clone().to_owned());
            functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"variable category".to_string()).unwrap().push("var".to_string());
            functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"spillover".to_string()).unwrap().push((stmt.len()-3).to_string());
            stack_height += stmt.len() as i32 -2;
        } else if &stmt[0] == "fdec" {
            if !in_func {
                in_func = true;
            }  else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot nest functions");
                exit(1);
            }
            funcs = gen_fnc_dec(&stmt[2], funcs);
            let args = &stmt.clone()[3..stmt.clone().len()].concat();
            let args = args.split(',').collect::<Vec<&str>>();
            let args = args[0..].iter().map(|&s| s.to_string()).collect::<Vec<String>>();
            functions.insert(stmt[2].clone(), HashMap::from(
                [
                    ("args".to_string(), HashMap::from([
                        ("int".to_string(), vec![]),
                        ("str".to_string(), vec![]),
                        ("bool".to_string(), vec![]),
                        ("all".to_string(), vec![]),
                    ])),
                    ("vars".to_string(), HashMap::from([
                        ("int".to_string(), vec![]),
                        ("str".to_string(), vec![]),
                        ("bool".to_string(), vec![]),
                        ("all".to_string(), vec![]),
                    ])),
                    ("stack".to_string(), HashMap::from([
                        ("stack height".to_string(), vec![String::from("-1")]),
                        ("variable name".to_string(), vec![String::from("N/A")]),
                        ("variable category".to_string(), vec![String::from("N/A")]),
                        ("spillover".to_string(), vec![String::from("0")]),
                    ])),
                    ("return value".to_string(), HashMap::from([
                        (stmt[1].clone(), vec![]),
                    ]))
                ]
            ));
            curr_func = stmt[2].clone();
            for arg in args {
                if arg != "" {
                    let arg = arg.split(":").collect::<Vec<&str>>();
                    let _type = arg[0];
                    let value = arg[1];
                    functions.get_mut(&stmt[2]).unwrap().get_mut(&"args".to_string()).unwrap().get_mut(&_type.to_string()).unwrap().push(value.to_string());
                    functions.get_mut(&stmt[2]).unwrap().get_mut(&"args".to_string()).unwrap().get_mut(&"all".to_string()).unwrap().push(value.to_string());
                    functions.get_mut(&stmt[2]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"stack height".to_string()).unwrap().push(stack_height.to_string());
                    functions.get_mut(&stmt[2]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"variable name".to_string()).unwrap().push(value.to_owned());
                    functions.get_mut(&stmt[2]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"variable category".to_string()).unwrap().push("arg".to_string());
                    functions.get_mut(&stmt[2]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"spillover".to_string()).unwrap().push("0".to_string());
                    stack_height += 1;
                }
            }
            if is_module {
                text.push(format!("    global {}", stmt[2].clone()));
            }
        } else if &stmt[0] == "endfunc" {
            funcs.push('\n'.to_string());
            curr_func = "".to_string();
            in_func = false;
            section = vec![]
        } else if &stmt[0] == "endif" {
            if stmts[i+1] == vec!["endif"] {
                let mut nested_if_counter: u32 = 1;
                while stmts[i+nested_if_counter as usize] == vec!["endif"] {
                    section.pop();
                    nested_if_counter+=1;
                    if_depth -= 1;
                }
                if vec!["else"].contains(&stmts[i+nested_if_counter as usize][0].as_str()) {
                    if stmts[i+nested_if_counter as usize][0] == "else" {
                        if stmts[i+nested_if_counter as usize].len() == 3 {
                            if if_statements[if_statements.len()-1].0 == "else if" {
                                if if_statements[if_statements.len()-1].2 {
                                    funcs.push(format!("    jmp .section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                                }
                                    funcs.push(format!(".section{}{}:", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+1).to_string()].join("_")));
                                section.pop();
                            } else {
                                if if_statements[if_statements.len()-1].2 {
                                    funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                                }
                                funcs.push(format!(".section{}{}:", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                            }
                        } else {
                            if if_statements[if_statements.len()-1].0 == "else if" {
                                if if_statements[if_statements.len()-1].2 {
                                    funcs.push(format!("    jmp .section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                                }
                                funcs.push(format!(".section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+1).to_string()].join("_")));
                                section.pop();
                            } else {
                                if if_statements[if_statements.len()-1].2 {
                                    funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                                }
                                funcs.push(format!(".section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                                
                            }
                        }
                    } else {
                        if if_statements[if_statements.len()-1].0 == "else if" {
                            if if_statements[if_statements.len()-1].2 {
                                funcs.push(format!("    jmp .section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                            }
                            funcs.push(format!(".section{}{}:", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+1).to_string()].join("_")));
                            section.pop();
                        } else {
                            if if_statements[if_statements.len()-1].2 {
                                funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                            }
                            funcs.push(format!(".section{}{}:", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                        }
                    }
                } else {
                    if if_statements[if_statements.len()-1].0 == "else if" {
                        if if_statements[if_statements.len()-1].2 {
                            funcs.push(format!("    jmp .section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                            funcs.push(format!(".section{}{}:", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+1).to_string()].join("_")));
                        }
                        section.pop();
                    } else {
                        if section.len() == 1 {
                            if if_statements[if_statements.len()-1].2 {
                                funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                                funcs.push(format!(".section{}{}:", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                            }
                        } else {
                            if if_statements[if_statements.len()-1].2 {
                                funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                                funcs.push(format!(".section{}{}:", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                            }
                        }
                    }
                }
                i += (nested_if_counter-1) as usize;
            } else if vec!["else"].contains(&stmts[i+1][0].as_str()) {
                if_depth -=1;
                if stmts[i+1][0] == "else" {
                    if stmts[i+1].len() == 3 {
                        if if_statements[if_statements.len()-1].0 == "else if" {
                            if if_statements[if_statements.len()-1].2 {
                                funcs.push(format!("    jmp .section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                            }
                                funcs.push(format!(".section{}{}:", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+1).to_string()].join("_")));
                            section.pop();
                        } else {
                            if if_statements[if_statements.len()-1].2 {
                                funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                            }
                            funcs.push(format!(".section{}{}:", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                        }
                    } else {
                        if if_statements[if_statements.len()-1].0 == "else if" {
                            if if_statements[if_statements.len()-1].2 {
                                funcs.push(format!("    jmp .section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                            }
                            funcs.push(format!(".section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+1).to_string()].join("_")));
                            section.pop();
                        } else {
                            if if_statements[if_statements.len()-1].2 {
                                funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                            }
                            funcs.push(format!(".section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                            
                        }
                    }
                } else {
                    if if_statements[if_statements.len()-1].0 == "else if" {
                        if if_statements[if_statements.len()-1].2 {
                            funcs.push(format!("    jmp .section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                        }
                        funcs.push(format!(".section{}{}:", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+1).to_string()].join("_")));
                        section.pop();
                    } else {
                        if if_statements[if_statements.len()-1].2 {
                            funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                        }
                        funcs.push(format!(".section{}{}:", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                    }
                }
            } else {
                if_depth -=1;
                if if_statements[if_statements.len()-1].0 == "else if" {
                    if if_statements[if_statements.len()-1].2 {
                        funcs.push(format!("    jmp .section{}{}", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                        funcs.push(format!(".section{}{}:", if section.len() > 2 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-2]+1).to_string()].join("_")));
                    }
                    section.pop();
                } else {
                    if section.len() == 1 {
                        if if_statements[if_statements.len()-1].2 {
                            funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                            funcs.push(format!(".section{}{}:", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-1].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                        }
                    } else {
                        if if_statements[if_statements.len()-1].2 {
                            funcs.push(format!("    jmp .section{}{}", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+if_statements[if_statements.len()-1].1).to_string()].join("_")));
                            funcs.push(format!(".section{}{}:", if section.len() > 1 {"_"} else {""}, [section[0..section.len()-2].iter().map(|i|i.to_string()).collect::<Vec<String>>().join("_"), (section[section.len()-1]+1).to_string()].join("_")));
                        }
                    }
                }
            }
        } else if &stmt[0] == "fcall"{
            if in_func {
                funcs = gen_fnc_call(&stmt[1], &stmt[2..], &vars, &curr_func, false, funcs);
            } else {
                start = gen_fnc_call(&stmt[1], &stmt[2..], &vars, &curr_func, false, start);
            }
            let arg_names = functions.get(&stmt[1]).unwrap().get("args").unwrap().get("all").unwrap().clone();

            for a_name in arg_names {
                if functions.get(&stmt[1]).unwrap().get(&"stack".to_string()).unwrap().get(&"variable name".to_string()).unwrap().contains(&a_name) {
                    
                }
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
                if vars.get("funcs").unwrap().get("str").unwrap().contains_key(&curr_func) {
                    if vars.get(&curr_func).unwrap().get("vars").unwrap().get("names").unwrap().contains(&stmt[1..][0])
                    ||vars.get(&curr_func).unwrap().get("args").unwrap().get("names").unwrap().contains(&stmt[1..][0]) {
                        let size = get_var_size(&vars.get_mut(&curr_func).unwrap().get_mut("stack").unwrap(), &stmt[1]);
                        vars.get_mut("funcs").unwrap().get_mut("str").unwrap().get_mut(&curr_func).unwrap().push(size.to_string());
                    } else if is_fnc_call(&stmt[1..][0]) {
                        let size = vars.get("funcs").unwrap().get("str").unwrap().get(&stmt[1]).unwrap()[0].clone();
                        vars.get_mut("funcs").unwrap().get_mut("str").unwrap().get_mut(&curr_func).unwrap().push(size.to_string());
                    } else {

                        vars.get_mut("funcs").unwrap().get_mut("str").unwrap().get_mut(&curr_func).unwrap().push((stmt.len()-1).to_string());
                    }
                }
            } else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: `ret` can only be used within a function.\nTo exit the program, use `out`");
                exit(1);
            }
        } else if &stmt[0] == "module" {
            is_module = true;
        } else if &stmt[0] == "if" {
            if in_func {
                if_statements.push(("if".to_string(), stmt[1].clone().parse::<i32>().unwrap(), if stmt[2] == "yes" {true} else {false}));
                if stmt.contains(&"=".to_string()) {
                    funcs = gen_if(&stmt[1].parse::<i32>().unwrap(), &section, &"=".to_string(), &stmt[3..stmt.iter().position(|n| n == &"=".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &"=".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                } else if stmt.contains(&">".to_string()) {
                    funcs = gen_if(&stmt[1].parse::<i32>().unwrap(), &section, &">".to_string(), &stmt[3..stmt.iter().position(|n| n == &">".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &">".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                } else if stmt.contains(&"<".to_string()) {
                    funcs = gen_if(&stmt[1].parse::<i32>().unwrap(), &section, &"<".to_string(), &stmt[3..stmt.iter().position(|n| n == &"<".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &"<".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                } else if stmt.contains(&">=".to_string()) {
                    funcs = gen_if(&stmt[1].parse::<i32>().unwrap(), &section, &">=".to_string(), &stmt[3..stmt.iter().position(|n| n == &">=".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &">=".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                } else if stmt.contains(&"<=".to_string()) {
                    funcs = gen_if(&stmt[1].parse::<i32>().unwrap(), &section, &"<=".to_string(), &stmt[3..stmt.iter().position(|n| n == &"<=".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &"<=".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                } else if stmt.contains(&"!".to_string()) {
                    funcs = gen_if(&stmt[1].parse::<i32>().unwrap(), &section, &"!".to_string(), &stmt[3..stmt.iter().position(|n| n == &"!".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &"!".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                }
            } else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: if statements cannot exist outside of a function scope");
                exit(1);
            }
            let section_len: usize = section.len();
            if if_depth >= section_len as u32 {
                section.push(1);
            } else {
                section[section_len-1]+=1;
            } 
            if_depth+=1;
        } else if &stmt[0] == "else" {
            if in_func {
                if_depth+=1;
                if &stmt[1] == "if" {
                    if_depth+=1;
                    let section_len: usize = section.len();
                    section[section_len-1]+=1;
                    if_statements.push(("else if".to_string(), stmt[2].clone().parse::<i32>().unwrap(), if stmt[3] == "yes" {true} else {false}));
                    if stmt.contains(&"=".to_string()) {
                        funcs = gen_if(&stmt[2].parse::<i32>().unwrap(), &section, &"=".to_string(), &stmt[4..stmt.iter().position(|n| n == &"=".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &"=".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                    } else if stmt.contains(&">".to_string()) {
                        funcs = gen_if(&stmt[2].parse::<i32>().unwrap(),&section, &">".to_string(), &stmt[4..stmt.iter().position(|n| n == &">".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &">".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                    } else if stmt.contains(&"<".to_string()) {
                        funcs = gen_if(&stmt[2].parse::<i32>().unwrap(),&section, &"<".to_string(), &stmt[4..stmt.iter().position(|n| n == &"<".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &"<".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                    } else if stmt.contains(&">=".to_string()) {
                        funcs = gen_if(&stmt[2].parse::<i32>().unwrap(),&section, &">=".to_string(), &stmt[4..stmt.iter().position(|n| n == &">=".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &">=".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                    } else if stmt.contains(&"<=".to_string()) {
                        funcs = gen_if(&stmt[2].parse::<i32>().unwrap(),&section, &"<=".to_string(), &stmt[4..stmt.iter().position(|n| n == &"<=".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &"<=".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                    } else if stmt.contains(&"!".to_string()) {
                        funcs = gen_if(&stmt[2].parse::<i32>().unwrap(),&section, &"!".to_string(), &stmt[4..stmt.iter().position(|n| n == &"!".to_string()).unwrap()], &stmt[stmt.iter().position(|n| n == &"!".to_string()).unwrap()+1..], (&constants, &functions), &curr_func, funcs);
                    }
                    section.push(1);
                } else {
                    if_statements.push(("else".to_string(), stmt[1].clone().parse::<i32>().unwrap(), if stmt[2] == "yes" {true} else {false}));
                    let section_len: usize = section.len();
                    section[section_len-1]+=1;
                }
            } else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: if statements cannot exist outside of a function scope");
                exit(1);
            }
        } else if stmt[0] == "require"{ 
            data = gen_require(&get_external_functions(extern_scopes), data);
        } else {
            println!("unknown statement, {stmt:?}");
            exit(11);
        }
        i+=1;
    }

    if !is_module {
        text.push("    global _start".to_string());
    }
    if !is_module {
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
    if !is_module {
        match writeln!(writer, "_start:") {
            Ok(_) => {},
            Err(_) => {
                eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
                exit(1);
            },
        }
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
