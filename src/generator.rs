

use std::{collections::HashMap, fs::File, io::{BufWriter, Write}, process::exit};

fn get_variable_type(var: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String) -> String {
    if is_variable_int(var, vars, curr_func) {
        return String::from("int");
    } else if is_variable_bool(var, vars, curr_func) {
        return String::from("bool");
    } else if is_variable_str(var, vars, curr_func) {
        return String::from("str");
    } else {
        return String::new()
    }
}

fn get_function_type(function: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>) -> String {
    return vars.get(function).unwrap().get("return").unwrap().keys().cloned().collect::<Vec<String>>()[0].clone();
}

fn get_var_size(var: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String) -> u32 {
    let mut size: u32 = 0;
    if is_int_lit(&var) 
    || is_variable_int(var, vars.1, curr_func)
    || is_int_function(var, vars.1){
        size += 4;
    } else if is_constant_int(var, vars.0)  {
        size += vars.0.get("int").unwrap().get("lengths").unwrap()[vars.0.get("int").unwrap().get("names").unwrap().iter().position(|s|s==var).unwrap()].parse::<u32>().unwrap();
    } else if matches!(var.as_str(), "True" | "False") ||  is_variable_bool(var, vars.1, curr_func)
    || is_constant_bool(var, vars.0) 
    || is_bool_function(var, vars.1){
        size +=1;
    } else if is_str_lit(&var) {
        size += var.chars().count() as u32 -2;
    } else if is_constant_str(var, vars.0)  {
        size += vars.0.get("str").unwrap().get("lengths").unwrap()[vars.0.get("str").unwrap().get("names").unwrap().iter().position(|s|s==var).unwrap()].parse::<u32>().unwrap();
    } else if is_str_function(var, vars.1) {
        size += vars.1.get(var).unwrap().get("return").unwrap().get("str").unwrap()[0].parse::<u32>().unwrap();
    } else if is_variable_str(var, vars.1, curr_func) {
        size += vars.1.get(var).unwrap().get("stack").unwrap().get("size").unwrap()[vars.1.get(curr_func).unwrap().get("stack").unwrap().get("names").unwrap().iter().position(|s|s==var).unwrap()].parse::<u32>().unwrap();
    }
    return size;
}

fn get_offset(var: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String) -> i32 {
    let stack = vars.get(curr_func).unwrap().get("stack").unwrap();
    let stack_height = get_stack_height(stack, &"all".to_string());
    let base =stack.get("stack height").unwrap()[stack.get("variable name").unwrap().iter().position(|v|v==var).unwrap()].parse::<i32>().unwrap(); 
    let mut offset = (stack_height - base-1)*8;
    if is_argument(var, vars, curr_func) {
        offset+=16+4096;
    }
    return offset;
}

fn is_int(tok: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String) -> bool {
    if is_int_lit(tok) || is_constant_int(tok, vars.0) || is_variable_int(tok, vars.1, curr_func) || 
    (is_fnc_call(tok) && tok.contains(&"fcall!") && is_int_function(&tok.strip_prefix("fcall!").unwrap().to_string().split("$").collect::<Vec<&str>>()[0].to_owned(), vars.1)) 
    || (is_fnc_call(tok) && tok.contains(&"fcall") && is_int_function(&tok.strip_prefix("fcall").unwrap().to_string().split("$").collect::<Vec<&str>>()[0].to_owned(), vars.1)){
        return true;
    } 
    return false;
}

fn is_bool(tok: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String) -> bool {
    if is_bool_lit(tok) || is_constant_bool(tok, vars.0) || is_variable_bool(tok, vars.1, curr_func)
    || (is_fnc_call(tok) && tok.contains(&"fcall!") && is_bool_function(&tok.strip_prefix("fcall!").unwrap().to_string().split("$").collect::<Vec<&str>>()[0].to_owned(), vars.1)) 
    || (is_fnc_call(tok) && tok.contains(&"fcall") && is_bool_function(&tok.strip_prefix("fcall").unwrap().to_string().split("$").collect::<Vec<&str>>()[0].to_owned(), vars.1)) {
        return true;
    }
    return false;
}

fn is_str(tok: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String) -> bool {
    if is_str_lit(tok) || is_constant_str(tok, vars.0) || is_variable_str(tok, vars.1, curr_func) || 
    (is_fnc_call(tok) && tok.contains(&"fcall!") && is_str_function(&tok.strip_prefix("fcall!").unwrap().to_string().split("$").collect::<Vec<&str>>()[0].to_owned(), vars.1)) 
    || (is_fnc_call(tok) && tok.contains(&"fcall") && is_str_function(&tok.strip_prefix("fcall").unwrap().to_string().split("$").collect::<Vec<&str>>()[0].to_owned(), vars.1)){
        return true;
    }
    return false;
}

fn is_bool_lit(tok: &String) -> bool {
    if matches!(tok.as_str(), "True" | "False") {
        return true;
    }
    return false;
}

fn is_function(function: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>) -> bool {
    if vars.keys().filter(|key|!is_int_lit(key)).collect::<Vec<&String>>().contains(&function) {
        return true;
    }
    return false;
}

fn is_int_function(function: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>) -> bool {
    if is_function(function, vars) {
        if vars.get(function).unwrap().get("return").unwrap().contains_key("int") {
            return true;
        }
        return false;
    } 
    return false;
}

fn is_str_function(function: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>) -> bool {
    if is_function(function, vars) {
        if vars.get(function).unwrap().get("return").unwrap().contains_key("str") {
            return true;
        }
        return false;
    } 
    return false;
}

fn is_bool_function(function: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>) -> bool {
    if is_function(function, vars) {
        if vars.get(function).unwrap().get("return").unwrap().contains_key("bool") {
            return true;
        }
        return false;
    } 
    return false;
}

fn is_constant(var: &String, constants: &HashMap<String, HashMap<String, Vec<String>>>) -> bool {
    if constants.get("all").unwrap().get("names").unwrap().contains(&var) {
        return true;
    }
    return false;
}

fn is_constant_int(var: &String, constants: &HashMap<String, HashMap<String, Vec<String>>>) -> bool {
    if is_constant(var, constants) {
        if constants.get("int").unwrap().get("names").unwrap().contains(var) {
            return true;
        } 
        return false;
    }
    return false;
}

fn is_constant_str(var: &String, constants: &HashMap<String, HashMap<String, Vec<String>>>) -> bool {
    if is_constant(var, constants) {
        if constants.get("str").unwrap().get("names").unwrap().contains(var) {
            return true;
        } 
        return false;
    }
    return false;
}

fn is_constant_bool(var: &String, constants: &HashMap<String, HashMap<String, Vec<String>>>) -> bool {
    if is_constant(var, constants) {
        if constants.get("bool").unwrap().get("names").unwrap().contains(var) {
            return true;
        } 
        return false;
    }
    return false;
}

fn scope_takes_arguments(vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String) -> bool {
    if vars.get(curr_func).unwrap().contains_key(&"args".to_string()) {
        return true;
    }
    return false;
}

fn is_argument(var: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String) -> bool {
    if scope_takes_arguments(vars, curr_func) {
        if vars.get(curr_func).unwrap().get("args").unwrap().get("all").unwrap().contains(var) {
            return true;
        }
        return false;
    }
    return false;
}

fn is_variable(var: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String) -> bool {
    if vars.get(curr_func).unwrap().get("vars").unwrap().get("all").unwrap().contains(var) {
        return true;
    } else {
        if scope_takes_arguments(vars, curr_func) {
            if vars.get(curr_func).unwrap().get("args").unwrap().get("all").unwrap().contains(var) {
                return true;
            }
            return false;
        }
        return false;
    }
}

fn is_variable_int(var: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String) -> bool {
    if is_variable(var, vars, curr_func) {
        if vars.get(curr_func).unwrap().get("vars").unwrap().get("int").unwrap().contains(var) {
            return true;
        } else {
            if scope_takes_arguments(vars, curr_func) {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("int").unwrap().contains(var) {
                    return true;
                }
                return false;
            }
            return false;
        } 
    }
    return false;
}

fn is_variable_str(var: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String) -> bool {
    if is_variable(var, vars, curr_func) {
        if vars.get(curr_func).unwrap().get("vars").unwrap().get("str").unwrap().contains(var) {
            return true;
        } else {
            if scope_takes_arguments(vars, curr_func) {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("str").unwrap().contains(var) {
                    return true;
                }
                return false;
            }
            return false;
        } 
    }
    return false;
}

fn is_variable_bool(var: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_func: &String) -> bool {
    if is_variable(var, vars, curr_func) {
        if vars.get(curr_func).unwrap().get("vars").unwrap().get("bool").unwrap().contains(var) {
            return true;
        } else {
            if scope_takes_arguments(vars, curr_func) {
                if vars.get(curr_func).unwrap().get("args").unwrap().get("bool").unwrap().contains(var) {
                    return true;
                }
                return false;
            }
            return false;
        } 
    }
    return false;
}

fn is_fnc_call(expr: &String) -> bool {
    if expr.contains("fcall") {
        return true;
    }
    return false;
}

fn is_str_lit(tok: &String) -> bool {
    if tok.chars().nth(0).unwrap() == '\"' {
        for i in 1..tok.chars().count() {
            let ch = tok.chars().nth(i).unwrap();
            if ch == '\"' && i != tok.chars().count()-1 {
                return false;
            }
        }
        return true;
    } else {
        return false;
    }
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

fn get_stack_height(stack: &HashMap<String, Vec<String>>, mode: &String) -> i32 {
    let mut height: i32 = 0;
    for i in (0..stack.get("stack height").unwrap().len()).rev() {
        if mode == "vars" {
            if stack.get("variable category").unwrap()[i] != "var" {
                continue;
            }
        }
        if mode == "args" {
            if stack.get("variable category").unwrap()[i] != "arg" {
                continue;
            }
        }
        height = stack.get("stack height").unwrap()[stack.get("stack height").unwrap().len()-1].parse::<i32>().unwrap() + (stack.get("size").unwrap()[stack.get("size").unwrap().len()-1].parse::<i32>().unwrap());
    }
    return height;
}


fn move_into_register_from_stack(reg: &String, offset: &u32, _size: &u32, type_: &String, mut asm: Vec<String>) -> Vec<String> {
    if type_ == "int" {
        asm.push(format!("    cqo"));
        asm.push(format!("    mov rax, [rsp+{offset}]"));

        asm.push(format!("    imul rax, rax, 16777216"));
        asm.push(format!("    mov r15, rax"));
        asm.push(format!("    mov rax, [rsp+{}]", offset-8));
        asm.push(format!("    imul rax, rax, 65536"));
        asm.push(format!("    add rax, r15"));
        asm.push(format!("    mov r15, rax"));
        asm.push(format!("    mov rax, [rsp+{}]", offset-16));
        asm.push(format!("    imul rax, rax, 256"));
        asm.push(format!("    add rax, r15"));
        asm.push(format!("    add rax, [rsp+{}]", offset-24));
        if reg != "rax" {
            asm.push(format!("    mov {reg}, rax"));
        }
        return asm;
    } else if type_ == "bool" {
        asm.push(format!("    mov {reg}, [rsp+{offset}]"));
    }

    return asm;
}

fn move_to_stack_from_register(reg: &String, _size: &u32, type_: &String, mut asm: Vec<String>) -> Vec<String>  {
    if type_ == "int" {
        if reg != "rax" {
            asm.push(format!("    mov rax, {reg}"));
        }
        asm.push(format!("    cqo"));
        asm.push(format!("    xor rdx, rdx"));
        asm.push(format!("    mov r15, 16777216"));
        asm.push(format!("    idiv r15"));
        asm.push(format!("    push rax"));
        asm.push(format!("    mov rax, rdx"));
        asm.push(format!("    xor rdx, rdx"));
        asm.push(format!("    mov r15, 65536"));
        asm.push(format!("    idiv r15"));
        asm.push(format!("    push rax"));
        asm.push(format!("    mov rax, rdx"));
        asm.push(format!("    xor rdx, rdx"));
        asm.push(format!("    mov r15, 256"));
        asm.push(format!("    idiv r15"));
        asm.push(format!("    push rax"));
        asm.push(format!("    push rdx"));
    }
    else if type_ == "bool" {
        asm.push(format!("    push {reg}"));
    }
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
        asm = move_into_register_from_stack(&"rbx".to_string(), &24, &4, &"int".to_string(), asm);
    } else if is_fnc_call(&lhs[0]) {
        asm = gen_fnc_call(&lhs[1], &lhs[2..], vars, curr_func, false, asm);
        asm.push(format!("    mov rbx, rax"));
    } else if is_variable(&lhs[0], vars.1, curr_func) {
        let offset = get_offset(&lhs[0], vars.1, curr_func);
        asm = move_into_register_from_stack(&"rbx".to_string(), &(offset as u32), &get_var_size(&lhs[0], vars, curr_func), &get_variable_type(&lhs[0], vars.1, curr_func), asm);
    } else if is_constant_int(&lhs[0],vars.0) 
    || is_constant_bool(&lhs[0],vars.0) {
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
        asm = move_into_register_from_stack(&"rax".to_string(), &24, &4, &"int".to_string(), asm);
    } else if is_fnc_call(&rhs[0]) {
        asm = gen_fnc_call(&rhs[1], &rhs[2..], vars, curr_func, false, asm);
    } else if is_variable(&rhs[0], vars.1, curr_func) {
        let offset = get_offset(&rhs[0], vars.1, curr_func);
        asm = move_into_register_from_stack(&"rax".to_string(), &(offset as u32), &get_var_size(&rhs[0], vars, curr_func), &get_variable_type(&rhs[0], vars.1, curr_func), asm);
    } else if is_constant_int(&rhs[0], vars.0) || is_constant_bool(&rhs[0], vars.0){
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
        asm.push(format!("    {name} dq {}", if value.concat() == "True" {1} else {0}));
    } else {
        println!("unknown constant type");
        exit(11);
    }
    return asm;
}

fn gen_printf(prf: &[String], fd: u8, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), print_section_c: &u32, curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    let mut section_count: u32 = 1;
    for term in prf {
        if is_str_lit(&term) {
            asm = gen_print(term, fd, asm);
        } else if is_str(&term, vars, curr_func) && !is_str_lit(&term) {
            if is_constant_str(&term, vars.0) {
                asm.push(format!("    mov rsi, [{term}]"));
            } else if is_variable_str(&term, vars.1, curr_func) {
                asm = move_into_register_from_stack(&"rax".to_string(), &(get_offset(term, vars.1, curr_func) as u32), &1, &"str".to_string(), asm);
            } else if is_fnc_call(&term) {
                let name = term.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
                let args = &term.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
                asm = gen_fnc_call(&name, &args, vars, curr_func, true,asm); 
            }
            asm.push(format!("    mov rax, 1"));
            asm.push(format!("    mov rdi, {fd}"));
            asm.push(format!("    mov rdx, {}", get_var_size(term, vars, curr_func)));
            asm.push(format!("    syscall"));
        } else if is_bool(term, vars, curr_func) {
            if is_constant_bool(&term, vars.0) {
                asm.push(format!("    mov rax, [{term}]"));
            } else if is_bool_lit(&term) {
                if term == "True" {
                    asm.push(format!("    mov rax, 1"));
                } else {
                    asm.push(format!("    mov rax, 0"));
                }
            } else if is_variable_bool(&term, vars.1, curr_func) {
                asm = move_into_register_from_stack(&"rax".to_string(), &(get_offset(term, vars.1, curr_func) as u32), &1, &"bool".to_string(), asm);
            } else if is_fnc_call(&term) {
                let name = term.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
                let args = &term.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
                asm = gen_fnc_call(&name, &args, vars, curr_func, true,asm); 
            }
            asm.push(format!("    mov rbx, 1"));
            asm.push(format!("    cmp rbx, rax"));
            asm.push(format!("    je .print_section_{print_section_c}_{section_count}"));
            asm.push(format!("    jne .print_section_{print_section_c}_{}", section_count+1));
            asm.push(format!(".print_section_{print_section_c}_{section_count}:"));
            asm = gen_print(&"True".to_string(), fd, asm);
            asm.push(format!("    jmp .print_section_{print_section_c}_{}", section_count+2));
            asm.push(format!(".print_section_{print_section_c}_{}:", section_count+1));
            asm = gen_print(&"False".to_string(), fd, asm);
            asm.push(format!(".print_section_{print_section_c}_{}:", section_count+2));
            section_count+=3;
        } else if is_int(term, vars, curr_func) {
            asm.push(format!("    mov r15, 0"));
            if is_constant_int(&term, vars.0) {
                asm.push(format!("    mov rax, [{term}]"));
            } else if is_int_lit(&term) {
                asm.push(format!("    mov rax, {term}"))
            } else if is_variable_int(term, vars.1, curr_func) {
                asm = move_into_register_from_stack(&"rax".to_string(), &(get_offset(term, vars.1, curr_func) as u32), &4, &"int".to_string(), asm);
            } else if is_fnc_call(&term) {
                let name = term.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
                let args = &term.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
                asm = gen_fnc_call(&name, &args, vars, curr_func, true,asm); 
            }   
            asm.push(format!("    mov rbx, 10"));
            asm.push(format!("    cmp rax, rbx"));
            asm.push(format!("    jl .print_section_{print_section_c}_{section_count}"));
            asm.push(format!("    jge .print_section_{print_section_c}_{}", section_count+1));
            asm.push(format!(".print_section_{print_section_c}_{section_count}:"));
            asm.push(format!("    add al, 48"));
            asm.push(format!("    push rax"));
            asm.push(format!("    mov rsi, rsp"));
            asm.push(format!("    mov rax, 1"));
            asm.push(format!("    mov rdi, {fd}"));    
            asm.push(format!("    mov rdx, 1"));
            asm.push(format!("    syscall"));
            asm.push(format!("    pop r8"));
            asm.push(format!("    jmp .print_section_{print_section_c}_{}", section_count+4));
            asm.push(format!(".print_section_{print_section_c}_{}:", section_count+1));
            asm.push(format!("    inc r15"));
            asm.push(format!("    cqo"));
            asm.push(format!("    mov rbx, 10"));
            asm.push(format!("    idiv rbx"));
            asm.push(format!("    push rdx"));
            asm.push(format!("    cmp rax, rbx"));
            asm.push(format!("    jge .print_section_{print_section_c}_{}", section_count+1));
            asm.push(format!("    jl .print_section_{print_section_c}_{}", section_count+2));
            asm.push(format!(".print_section_{print_section_c}_{}:", section_count+2));
            asm.push(format!("    add al, 48"));
            asm.push(format!("    push rax"));
            asm.push(format!("    mov rsi, rsp"));
            asm.push(format!("    mov rax, 1"));
            asm.push(format!("    mov rdi, {fd}"));
            asm.push(format!("    mov rdx, 1"));
            asm.push(format!("    syscall"));
            asm.push(format!("    pop r8"));
            asm.push(format!("    cmp r15, 0"));
            asm.push(format!("    jg .print_section_{print_section_c}_{}", section_count+3));
            asm.push(format!("    je .print_section_{print_section_c}_{}", section_count+4));
            asm.push(format!(".print_section_{print_section_c}_{}:", section_count+3));
            asm.push(format!("    dec r15"));
            asm.push(format!("    pop rax"));
            asm.push(format!("    add al, 48"));
            asm.push(format!("    push rax"));
            asm.push(format!("    mov rsi, rsp"));
            asm.push(format!("    mov rax, 1"));
            asm.push(format!("    mov rdi, {fd}"));
            asm.push(format!("    mov rdx, 1"));
            asm.push(format!("    syscall"));
            asm.push(format!("    cmp r15, 0"));
            asm.push(format!("    pop r8"));
            asm.push(format!("    jg .print_section_{print_section_c}_{}", section_count+3));
            asm.push(format!("    je .print_section_{print_section_c}_{}", section_count+4));
            asm.push(format!(".print_section_{print_section_c}_{}:", section_count+4));
            section_count+=4;
            asm = gen_print_newline(fd, asm);
        }
    }
    return asm;

}

fn gen_fnc_call(name: &String, args: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if is_function(name, vars.1) {
        let f = vars.1.get(name).unwrap();
        let mut args = args.to_vec();
        if args.len() == 1 && args[0] == "" {
            args.pop();
        }
        if args.len() == f.get(&"args".to_string()).unwrap().get("all").unwrap().len(){
            if is_str_function(name, vars.1) {
                asm.push("    sub rsp, 512".to_string());
                asm.push("    lea rdi, [rsp]".to_string());
            }
            if args.len() > 0 {
                for i in 0..args.len() {
                    let arg = &args[i];
                    if is_int_lit(arg) {
                        asm.push(format!("    mov r15, {arg}"));
                        asm = move_to_stack_from_register(&"r15".to_string(), &4, &"int".to_string(), asm);
                    } else if is_variable(&arg, vars.1, curr_func) {
                        let mut offset = get_offset(&arg, vars.1, curr_func) + (i as i32)*8;
                        if in_print {
                            offset += 8;
                        }
                        asm = move_into_register_from_stack(&"r15".to_string(), &(offset as u32), &get_var_size(&arg, vars, curr_func), &get_variable_type(&arg, vars.1, curr_func), asm);
                        asm = move_to_stack_from_register(&"r15".to_string(), &get_var_size(&arg, vars, curr_func), &get_variable_type(&arg, vars.1, curr_func), asm);
                    } else if is_fnc_call(arg) {
                        eprintln!("\x1b[1mSyntaxError\x1b[0m: Functions cannot act as arguments to other functions");
                        eprintln!("Try using a variable with the same value instead");
                        exit(1);
                    } else if is_constant(arg, vars.0) {
                        asm.push(format!("    push {arg}"));
                    } else if matches!(arg.as_str(), "True" | "False") {
                        match arg.as_str() {
                            "True" => {asm.push(format!("    mov r15, 1")); asm.push(format!("    push r15"));},
                            _ => {asm.push(format!("    push r15")); asm.push(format!("    push r15"))},
                        }
                    }
                    else {
                        eprintln!("\x1b[1mParserError\x1b[0m: Type mismatch");
                        exit(1);
                    }
                }
            }
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Expected {} arguments in function `{}`, but got {}", f.get(&"args".to_string()).unwrap().get("all").unwrap().len(), name, args.len());
            exit(1);
        }
        asm.push(format!("    call {}", name));
        if is_str_function(name, vars.1) && !in_print {
            asm.push(format!("    add rsp, 512"));
        }
        for arg in args {
            asm.push(format!("    add rsp, {}", get_var_size(&arg, vars, curr_func)*8));
        }
    } else {
        eprintln!("\x1b[1mParserError\x1b[0m: Used undefined function `{}`", name);
    }
    return asm;
}

fn gen_ret(ret_val: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, mut asm: Vec<String>) -> Vec<String> {
    if is_int_function(curr_func, vars.1) {
        if has_bin_expr(&ret_val) {
            asm = gen_bin_expr(ret_val, vars, curr_func, false, asm);
            asm = move_into_register_from_stack(&"rax".to_string(), &24, &4, &"int".to_string(), asm);
        } else if is_fnc_call(&ret_val[0]) {
            let args: Vec<String> = ret_val[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&ret_val[1].to_string(), &args[0..], vars, curr_func, false, asm);
        } else if is_variable_int(&ret_val[0],vars.1, curr_func){
            let offset = get_offset(&ret_val[0], vars.1, curr_func);
            asm = move_into_register_from_stack(&"rax".to_string(), &(offset as u32), &get_var_size(&ret_val[0], vars, curr_func), &get_variable_type(&ret_val[0], vars.1, curr_func), asm);
        } else {
            asm.push(format!("    mov rax, {}", ret_val[0]));
        }
    } else if is_str_function(&curr_func, vars.1) {
        if is_variable_str(&ret_val[0], vars.1, curr_func) {
            let offset = get_offset(&ret_val[0], vars.1, curr_func);
            for i in 0..get_var_size(&ret_val[0], vars, curr_func) {
                asm.push(format!("    mov al, [rsp+{}]", offset - (i as i32 *8)));
                asm.push(format!("    mov [rdi+{}], al", i));
            }
        } else {
            for i in 0..ret_val.len() {
                asm.push(format!("    mov byte [rdi+{}], '{}'", i, ret_val[i]));
            }
            asm.push(format!("    mov byte [rdi+{}], 0", ret_val.len()));
        }
    } else if is_bool_function(curr_func, vars.1) {
        if is_fnc_call(&ret_val[0]) {
            let args: Vec<String> = ret_val[2..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&ret_val[1].to_string(), &args[0..], vars, curr_func, false, asm);
        } else if is_variable_bool(&ret_val[0], vars.1, curr_func) {
            let offset = get_offset(&ret_val[0], vars.1, curr_func);
            asm = move_into_register_from_stack(&"rax".to_string(), &(offset as u32), &get_var_size(&ret_val[0], vars, curr_func), &get_variable_type(&ret_val[0], vars.1, curr_func), asm);
        } else {
            match ret_val[0].as_str() {
                "True" => asm.push(format!("    mov rax, 1")),
                _ => asm.push(format!("    mov rax, 0")),
            }
        }
    }
    asm.push(format!("    mov rsp, rbp"));
    asm.push(format!("    pop rbp"));
    asm.push("    ret".to_string());

    return asm;
}

fn gen_fnc_dec(name: &String, mut asm: Vec<String>) -> Vec<String> {
    asm.push(format!("{name}:"));
    asm.push(format!("    push rbp"));
    asm.push(format!("    mov rbp, rsp"));
    asm.push(format!("    sub rsp, 4096"));
    return asm;
}

fn gen_print_newline(fd: u8, mut asm: Vec<String>) -> Vec<String> {
    asm.push(format!("    mov rax, 1"));
    asm.push(format!("    mov rdi, {fd}"));
    asm.push(format!("    mov byte [rsp], 10"));
    asm.push(format!("    mov rsi, rsp"));
    asm.push(format!("    mov rdx, 1"));
    asm.push(format!("    syscall"));
    asm.push(format!("    add rsp, 8"));
    return asm;
}

fn gen_print(prt: &String, fd: u8, mut asm: Vec<String>) -> Vec<String> {
    for ch in prt.chars() {
        if ch == '"' {
            continue;
        } else {
            asm.push(format!("    mov rax, 1"));
            asm.push(format!("    mov rdi, {fd}"));
            asm.push(format!("    mov byte [rsp], '{ch}'"));
            asm.push(format!("    mov rsi, rsp"));
            asm.push(format!("    mov rdx, 1"));
            asm.push(format!("    syscall"));
            asm.push(format!("    add rsp, 8"));
        }
    }
    asm = gen_print_newline(fd, asm);
    return asm;
}

fn gen_add(lhs: &String, rhs: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if !is_int_lit(rhs) {
        if rhs == "STACK" {
            asm = move_into_register_from_stack(&"rbx".to_string(), &24, &4, &"int".to_string(), asm);
        } else if is_variable(&rhs, vars.1, curr_func) {
            let mut offset = get_offset(&rhs, vars.1, curr_func);
            if in_print {
                offset += 8;
            }
            asm = move_into_register_from_stack(&"rbx".to_string(), &(offset as u32), &get_var_size(&rhs, vars, curr_func), &get_variable_type(&rhs, vars.1, curr_func), asm);
        } else if rhs.contains("fcall") {
            let name = rhs.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
            let args = &rhs.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
            asm = gen_fnc_call(&name, &args, vars, curr_func, in_print,asm);
            asm.push("mov rbx, rax".to_string());
        } else {
            asm.push(format!("    mov rbx, {}", rhs));
        }
    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }

    if !is_int_lit(lhs) {
        if lhs == "STACK" {
            asm = move_into_register_from_stack(&"rax".to_string(), &24, &4, &"int".to_string(), asm);
        } 
        else if is_variable(&lhs, vars.1, curr_func) {
            let mut offset = get_offset(&lhs, vars.1, curr_func);
            if in_print {
                offset += 8;
            }
            asm = move_into_register_from_stack(&"rax".to_string(), &(offset as u32), &get_var_size(&rhs, vars, curr_func), &get_variable_type(&rhs, vars.1, curr_func), asm);
        } else if lhs.contains("fcall") {
            let name = lhs.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
            let args = &lhs.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
            asm = gen_fnc_call(&name, &args, vars, curr_func, in_print,asm);
        } else {
            asm.push(format!("    mov rax, {}", lhs));
        }
    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    add rbx, rax".to_string());
    asm = move_to_stack_from_register(&"rbx".to_string(), &4, &"int".to_string(), asm);

    return asm;
}

fn gen_sub(lhs: &String, rhs: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    asm.push(format!("    cqo"));
    if !is_int_lit(rhs) {
        if rhs == "STACK" {
            asm = move_into_register_from_stack(&"rbx".to_string(), &24, &4, &"int".to_string(), asm);
        } else if is_variable(&rhs, vars.1, curr_func) {
            let mut offset = get_offset(&rhs, vars.1, curr_func);
            if in_print {
                offset += 8;
            }
            asm = move_into_register_from_stack(&"rbx".to_string(), &(offset as u32), &get_var_size(&rhs, vars, curr_func), &get_variable_type(&rhs, vars.1, curr_func), asm);
        } else if rhs.contains("fcall") {
            let name = rhs.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
            let args = &rhs.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
            asm = gen_fnc_call(&name, &args, vars, curr_func, in_print,asm);
            asm.push("mov rbx, rax".to_string());
        } else {
            asm.push(format!("    mov rbx, {}", rhs));
        }
    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }
    if !is_int_lit(lhs) {
        if lhs == "STACK" {
            asm = move_into_register_from_stack(&"rax".to_string(), &24, &4, &"int".to_string(), asm);
        }
        else if is_variable(&lhs, vars.1, curr_func) {
            let mut offset = get_offset(&lhs, vars.1, curr_func);
            if in_print {
                offset += 8;
            }
            asm = move_into_register_from_stack(&"rax".to_string(), &(offset as u32), &get_var_size(&rhs, vars, curr_func), &get_variable_type(&rhs, vars.1, curr_func), asm);
        }  else if lhs.contains("fcall") {
            let name = lhs.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
            let args = &lhs.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
            asm = gen_fnc_call(&name, &args, vars, curr_func, in_print,asm);
        } else {
            asm.push(format!("    mov rax, {}", lhs));
        }
    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push("    sub rax, rbx".to_string());
    asm = move_to_stack_from_register(&"rax".to_string(), &4, &"int".to_string(), asm);

    return asm;
}

fn gen_mul(lhs: &String, rhs: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "STACK" || !is_int_lit(rhs) {
        if is_variable(&rhs, vars.1, curr_func){
            let mut offset = get_offset(&rhs, vars.1, curr_func);
            if in_print {
                offset += 8;
            }
            asm = move_into_register_from_stack(&"rbx".to_string(), &(offset as u32), &get_var_size(&rhs, vars, curr_func), &get_variable_type(&rhs, vars.1, curr_func), asm);
        }  else if rhs.contains("fcall") {
            let name = rhs.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
            let args = &rhs.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
            asm = gen_fnc_call(&name, &args, vars, curr_func, in_print,asm);
            asm.push("mov rbx, rax".to_string());
        } else {
            if rhs == "STACK" {
                asm = move_into_register_from_stack(&"rbx".to_string(), &24, &4, &"int".to_string(), asm);
            } else {
                asm.push(format!("    mov rbx, {}", rhs));
            }
        }

    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }
    if lhs == "STACK" || !is_int_lit(lhs) {
        if is_variable(&lhs, vars.1, curr_func) {
            let mut offset = get_offset(&lhs, vars.1, curr_func);
            if in_print {
                offset += 8;
            }
            asm = move_into_register_from_stack(&"rax".to_string(), &(offset as u32), &get_var_size(&rhs, vars, curr_func), &get_variable_type(&rhs, vars.1, curr_func), asm);
        }  else if lhs.contains("fcall") {
            let name = lhs.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
            let args = &lhs.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
            asm = gen_fnc_call(&name, &args, vars, curr_func, in_print,asm);
        } else {
            if lhs == "STACK" {
                asm = move_into_register_from_stack(&"rax".to_string(), &24, &4, &"int".to_string(), asm);
            } else {
                asm.push(format!("    mov rax, {}", lhs));
            }
        }

    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push(format!("    cqo"));
    asm.push(format!("    xor rdx, rdx"));
    asm.push("    imul rbx".to_string());
    asm = move_to_stack_from_register(&"rax".to_string(), &4, &"int".to_string(), asm);
    return asm;
}

fn gen_div(lhs: &String, rhs: &String, vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if rhs == "STACK" || !is_int_lit(rhs) {
        if is_variable(&rhs, vars.1, curr_func) {
            let mut offset = get_offset(&rhs, vars.1, curr_func);
            if in_print {
                offset += 8;
            }
            asm = move_into_register_from_stack(&"rbx".to_string(), &(offset as u32), &get_var_size(&rhs, vars, curr_func), &get_variable_type(&rhs, vars.1, curr_func), asm);
        }  else if rhs.contains("fcall") {
            let name = rhs.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
            let args = &rhs.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
            asm = gen_fnc_call(&name, &args, vars, curr_func, in_print,asm);
            asm.push("mov rbx, rax".to_string());
        } else {
            if rhs == "STACK" {
                asm = move_into_register_from_stack(&"rbx".to_string(), &24, &4, &"int".to_string(), asm);
                asm.push("    pop rbx".to_string());
            } else {
                asm.push(format!("    mov rbx, {}", rhs));
            }
        }

    } else {
        asm.push(format!("    mov rbx, {}", rhs));
    }
    if lhs == "STACK" || !is_int_lit(lhs) {
        if is_variable(&lhs, vars.1, curr_func) {
            let mut offset = get_offset(&lhs, vars.1, curr_func);
            if in_print {
                offset += 8;
            }
            asm = move_into_register_from_stack(&"rax".to_string(), &(offset as u32), &get_var_size(&rhs, vars, curr_func), &get_variable_type(&rhs, vars.1, curr_func), asm);
        }  else if lhs.contains("fcall") {
            let name = lhs.clone().split("!").collect::<Vec<&str>>()[1].split("$").collect::<Vec<&str>>()[0].to_string();
            let args = &lhs.clone().strip_prefix(&format!("fcall!{name}$")).unwrap().split("|").collect::<Vec<&str>>().iter().map(|s|s.to_string()).collect::<Vec<String>>()[0..];
            asm = gen_fnc_call(&name, &args, vars, curr_func, in_print,asm);
        } else {
            if lhs == "STACK" {
                asm = move_into_register_from_stack(&"rax".to_string(), &24, &4, &"int".to_string(), asm);
            } else {
                asm.push(format!("    mov rax, {}", lhs));
            }
        }

    } else {
        asm.push(format!("    mov rax, {}", lhs));
    }
    asm.push(format!("    cqo"));
    asm.push(format!("    xor rdx, rdx"));
    asm.push("    idiv rbx".to_string());
    asm = move_to_stack_from_register(&"rax".to_string(), &4, &"int".to_string(), asm);

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
            asm = move_into_register_from_stack(&"rdi".to_string(), &24, &4, &"int".to_string(), asm);
        } else {
            println!("improper out size?");
            exit(11);
        }
    } else {
        if is_bin_expr(&out_val[0]) {
            asm = gen_bin_expr(out_val, vars, curr_func, false, asm);
            asm = move_into_register_from_stack(&"rdi".to_string(), &24, &4, &"int".to_string(), asm);
        } else if is_fnc_call(&out_val[0]) {
            let args: Vec<String> = out_val[1..].iter().map(|s| s.to_string()).collect();
            asm = gen_fnc_call(&out_val[1].to_string(), &args[0..], vars, curr_func, false, asm);
            asm.push("     mov rdi, rax".to_string());
        } else if is_int_lit(&out_val[0]) {
            asm.push(format!("    mov rdi, {}", out_val[0]));
        } else if is_variable(&out_val[0], vars.1, curr_func) {
            let offset = get_offset(&out_val[0], vars.1, curr_func);
            asm = move_into_register_from_stack(&"rdi".to_string(), &(offset as u32), &get_var_size(&out_val[0], vars, curr_func), &get_variable_type(&out_val[0], vars.1, curr_func), asm)
        } else {
            asm.push(format!("    mov rdi, {}", out_val[0]));
        }
    }
    asm.push("    mov rax, 60".to_string());
    asm.push("    syscall".to_string());

    return asm;
}

fn gen_var_var(var_val: &[String], vars: (&HashMap<String, HashMap<String, Vec<String>>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_func: &String, in_print: bool, mut asm: Vec<String>) -> Vec<String> {
    if has_bin_expr(var_val) { 
        asm = gen_bin_expr(var_val, vars, curr_func, in_print, asm);
    } else if is_fnc_call(&var_val[0]) {
        let args: Vec<String> = var_val[2..].to_vec();
        asm = gen_fnc_call(&var_val[1].to_string(),&args[0..], vars, curr_func, false, asm);
        asm = move_to_stack_from_register(&"rax".to_string(), &get_var_size(&var_val[1].to_string(), vars, curr_func), &get_function_type(&var_val[1].to_string(), vars.1), asm)
    } else if is_variable(&var_val[0], vars.1, curr_func) {
        let offset = get_offset(&var_val[0], vars.1, curr_func);
        asm = move_into_register_from_stack(&"rax".to_string(), &(offset as u32), &get_var_size(&var_val[0].to_string(), vars, curr_func), &get_variable_type(&var_val[0], vars.1, curr_func), asm);
        asm = move_to_stack_from_register(&"rax".to_string(), &get_var_size(&var_val[0].to_string(), vars, curr_func), &get_variable_type(&var_val[0], vars.1, curr_func), asm)
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
        functions.extend(scope.to_owned());
    }
    let mut print_section_c: u32 = 1;
    let mut if_statements: Vec<(String, i32, bool)> = vec![];
    let mut if_depth: u32 = 0;
    let mut section: Vec<i32> = vec![];
    let mut is_module: bool = false;
    let mut curr_func: String = String::from("");
    let mut stack_height: i32 = 0;
    let mut fname: String = name.clone();
    if name.contains('/') {
        if let Some(index) = name.rfind('/') {
            fname = name[index+1..].to_string();
        }
    }
    let asm_name = build_dir.clone() + &fname.to_owned() + ".asm";
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
                    let size = get_var_size(&stmt[4..].concat(), (&constants, &functions), &curr_func);
                    constants.get_mut(&stmt[2]).unwrap().get_mut(&"lengths".to_string()).unwrap().push(size.to_string());
                    constants.get_mut("all").unwrap().get_mut(&"lengths".to_string()).unwrap().push(size.to_string());
                    constants.get_mut(&stmt[2]).unwrap().get_mut(&"names".to_string()).unwrap().push(stmt[3].clone());
                    constants.get_mut("all").unwrap().get_mut(&"names".to_string()).unwrap().push(stmt[3].clone());
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
                    let size = get_var_size(&stmt[4..].concat(), (&constants, &functions), &curr_func);
                    functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"size".to_string()).unwrap().push(size.to_string());
                    
                    stack_height += get_var_size(&stmt[2], (&constants, &functions), &curr_func) as i32 -1;
                }
        } else if &stmt[0] == &"print".to_string() {
            if in_func {
                funcs = gen_print(&stmt[1], 1, funcs);
            } else {
                start = gen_print(&stmt[1], 1, start);
            }
        } else if &stmt[0] == &"printf".to_string() {
            if in_func {
                funcs = gen_printf(&stmt[1..], 1, (&constants, &functions), &print_section_c, &curr_func, funcs);
            } else {
                start = gen_printf(&stmt[1..], 1, (&constants, &functions), &print_section_c, &curr_func, start);
            }
            print_section_c +=1;
        } else if &stmt[0] == &"eprint".to_string() {
            if in_func {
                funcs = gen_print(&stmt[1], 2, funcs);
            } else {
                start = gen_print(&stmt[1], 2, start);
            }
        } else if &stmt[0] == &"eprintf".to_string() {
            if in_func {
                funcs = gen_printf(&stmt[1..], 2, (&constants, &functions), &print_section_c, &curr_func, funcs);
            } else {
                start = gen_printf(&stmt[1..], 2, (&constants, &functions), &print_section_c, &curr_func, start);
            }
            print_section_c +=1;
        } else if &stmt[0] == &"vass".to_string() {
            if in_func {
                funcs = gen_var_var(&stmt[2..], (&constants, &functions), &curr_func,false, funcs);
            } else {
                start = gen_var_var(&stmt[2..], (&constants, &functions), &curr_func,false, start);
            }
            functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"stack height".to_string()).unwrap().push(stack_height.to_string());
            functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"variable name".to_string()).unwrap().push(stmt[3].clone().to_owned());
            functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"variable category".to_string()).unwrap().push("var".to_string());
            functions.get_mut(&curr_func).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"size".to_string()).unwrap().push((stmt.len()-3).to_string());
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
                        ("size".to_string(), vec![String::from("0")]),
                    ])),
                    ("return".to_string(), HashMap::from([
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
                    if _type == "int" {
                        functions.get_mut(&stmt[2]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"size".to_string()).unwrap().push("4".to_string());
                    } else if _type == "bool" {
                        functions.get_mut(&stmt[2]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&"size".to_string()).unwrap().push("1".to_string());
                    }
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
                funcs = gen_fnc_call(&stmt[1], &stmt[2..], (&constants, &functions), &curr_func, false, funcs);
            } else {
                start = gen_fnc_call(&stmt[1], &stmt[2..], (&constants, &functions), &curr_func, false, start);
            }
            let arg_names = functions.get(&stmt[1]).unwrap().get("args").unwrap().get("all").unwrap().clone();
            if functions.get(&stmt[1]).unwrap().contains_key(&"stack".to_string()) {
                for a_name in arg_names {
                    if functions.get(&stmt[1]).unwrap().get(&"args".to_string()).unwrap().contains_key(&a_name) {
                        functions.get_mut(&stmt[1]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&a_name).unwrap().remove(0);
                        functions.get_mut(&stmt[1]).unwrap().get_mut(&"stack".to_string()).unwrap().get_mut(&a_name).unwrap().push(stack_height.to_string());
                        stack_height += 1; 
                    } else {
                        functions.get_mut(&stmt[1]).unwrap().get_mut(&"stack".to_string()).unwrap().insert(
                            a_name.to_string(),
                            vec![stack_height.to_string()]
                        );
                        stack_height += 1;
                    }
                }
            }
        } else if &stmt[0] == "ret" {
            if in_func {
                funcs = gen_ret(&stmt[1..], (&constants, &functions), &curr_func,funcs);
                let return_type = &functions.get(&curr_func).unwrap().get("return").unwrap().keys().collect::<Vec<&String>>()[0];
                let size = get_var_size( &stmt[1..].concat(), (&constants, &functions), &curr_func);
                match return_type.as_str() {
                    "str" => {functions.get_mut(&curr_func).unwrap().get_mut("return").unwrap().get_mut("str").unwrap().push(size.to_string());},
                    "int" => {functions.get_mut(&curr_func).unwrap().get_mut("return").unwrap().get_mut("int").unwrap().push(size.to_string());},
                    "bool" => {functions.get_mut(&curr_func).unwrap().get_mut("return").unwrap().get_mut("bool").unwrap().push(size.to_string());},
                    _ => {eprintln!("\x1b[1mSyntaxError\x1b[0m: Unknown return type `{return_type}`"); exit(1);},
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

    match writeln!(writer, "default rel") {
        Ok(_) => {},
        Err(_) => {
            eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
            exit(1);
        },
    }
    if !is_module {
        text.push("    global _start".to_string());
    }
    match writeln!(writer, "section .data") {
        Ok(_) => {},
        Err(_) => {
            eprintln!("\x1b[1mGenerationError\x1b[0m: Failed to write to assembly file");
            exit(1);
        },
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
