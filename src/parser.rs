use std::{collections::HashMap, process::exit};

const TYPES: [&str; 4] = ["int", "vec", "str", "bool"];
const ATTRIBUTES: [&str; 2] = ["var", "const"];

fn parse_if(branch: &String, condition: &String, lhs: &[String], rhs: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_scope: &String) -> Vec<String> {
    let mut stmt =vec![];
    if branch == "i" {
        stmt = vec!["if".to_string()];
    } else if branch == "ei" {
        stmt = vec!["else".to_string(), "if".to_string()];
    }   
    let mut lhs_type = "";
    if is_bin_expr(lhs) && !lhs.contains(&"PARO".to_string()) {
        stmt = parse_bin_expr(lhs, &"var".to_string(), stmt);
        lhs_type = "int";
    } else if lhs.len() > 1 && lhs[1] == "PARO"{
        if vars.get("funcs").unwrap().get("bool").unwrap().contains_key(&lhs[0]) {
            stmt = parse_fnc_call(&lhs[0], &lhs[2..lhs.len()-1], vars, curr_scope, stmt);
            lhs_type = "bool";
        } else if vars.get("funcs").unwrap().get("int").unwrap().contains_key(&lhs[0]) {
            stmt = parse_fnc_call(&lhs[0], &lhs[2..lhs.len()-1], vars, curr_scope, stmt);
            lhs_type = "int";
        }
    } else if vars.get(curr_scope).unwrap().get("vars").unwrap().get("int").unwrap().contains(&lhs[0])
    || vars.get(curr_scope).unwrap().get("args").unwrap().get("int").unwrap().contains(&lhs[0]) 
    || vars.get("const").unwrap().get("int").unwrap().contains_key(&lhs[0]) || is_int_lit(&lhs[0]) {
        stmt.push(lhs[0].clone());
        lhs_type = "int";
    } else if vars.get(curr_scope).unwrap().get("vars").unwrap().get("bool").unwrap().contains(&lhs[0])
    || vars.get(curr_scope).unwrap().get("args").unwrap().get("bool").unwrap().contains(&lhs[0])
    || vars.get("const").unwrap().get("bool").unwrap().contains_key(&lhs[0]) {
        stmt.push(lhs[0].clone());
        lhs_type = "bool";
    } else if matches!(lhs[0].as_str(), "True" | "False")  {
        stmt.push(lhs[0].clone());
        lhs_type = "bool";
    } else {
        eprintln!("could not parse boolean lhs: {lhs:?}");
        exit(11);
    }
    stmt.push(condition.clone());
    if lhs_type == "int" {
        if is_bin_expr(rhs) {
            stmt = parse_bin_expr(rhs, &String::from("var"), stmt);
        } else if vars.get("funcs").unwrap().get("int").unwrap().contains_key(&rhs[0]) {
            stmt = parse_fnc_call(&rhs[0], &rhs[2..rhs.len()-1], vars, curr_scope, stmt);
        } else if vars.get(curr_scope).unwrap().get("vars").unwrap().get("int").unwrap().contains(&rhs[0])
        || vars.get(curr_scope).unwrap().get("args").unwrap().get("int").unwrap().contains(&rhs[0]) 
        || vars.get("const").unwrap().get("int").unwrap().contains_key(&rhs[0]) || is_int_lit(&rhs[0]) {
            stmt.push(rhs[0].clone());
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    } else if lhs_type == "bool" {
        if is_bin_expr(rhs) {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        } else if vars.get("funcs").unwrap().get("bool").unwrap().contains_key(&rhs[0]) {
            stmt = parse_fnc_call(&rhs[0], &rhs[2..rhs.len()-1], vars, curr_scope, stmt);
        } else if vars.get(curr_scope).unwrap().get("vars").unwrap().get("bool").unwrap().contains(&rhs[0])
        || vars.get(curr_scope).unwrap().get("args").unwrap().get("int").unwrap().contains(&rhs[0]) 
        || vars.get("const").unwrap().get("bool").unwrap().contains_key(&rhs[0]) {
            stmt.push(rhs[0].clone());
        } else if matches!(rhs[0].as_str(), "True" | "False")  {
            stmt.push(rhs[0].clone());
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    } else {
        eprintln!("unknown lhs_type");
        exit(11);
    }
    return stmt;
}

fn parse_bool(var: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, attribute: &String, curr_scope: &String, mut stmt:Vec<String>) -> Vec<String> {
    if var.len() > 1 {
        if is_bin_expr(var) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use booleans in binary expressions");
            exit(1);
        } else if var[1] == "PARO" {
            if attribute == "const" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use functions in constant variables");
                exit(1);
            }
            if vars.get("funcs").unwrap().get("bool").unwrap().contains_key(&var[0]) {
                stmt = parse_fnc_call(&var[0], &var[2..var.len()-1], vars, curr_scope, stmt);
            } else {
                eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
                exit(1);
            }
        }
    } else {
        if matches!(var[0].as_str(), "True" | "False") {
            stmt.push(var[0].clone());
        } else if vars.get(curr_scope).unwrap().get("args").unwrap().get("bool").unwrap().contains(&var[0])
        || vars.get(curr_scope).unwrap().get("vars").unwrap().get("bool").unwrap().contains(&var[0])
        || vars.get("const").unwrap().get("bool").unwrap().contains_key(&var[0]){
            stmt.push(var[0].clone()); 
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    }
    return stmt;
}

fn parse_str(var: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, attribute: &String, curr_scope: &String, mut stmt:Vec<String>) -> Vec<String> {
    if var.len() > 1 {
        if is_bin_expr(var) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use strings in binary expressions");
            exit(1);
        } else if var[1] == "PARO" {
            if attribute == "const" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use functions in constant variables");
                exit(1);
            }
            if vars.get("funcs").unwrap().get("str").unwrap().contains_key(&var[0]) {
                stmt = parse_fnc_call(&var[0], &var[2..var.len()-1], vars, curr_scope, stmt);
            } else {
                eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
                exit(1);
            }
        } else {
            exit(1);
        }
    } else {
        if is_string_lit(&var[0]) {
            if attribute == "var" {
                for ch in var[0].chars() {
                    if ch != '"' {
                        stmt.push(ch.to_string());
                    }
                }
            } else if attribute == "const" {
                stmt.push(var[0].clone());
            }
        } else if vars.get(curr_scope).unwrap().get("args").unwrap().get("str").unwrap().contains(&var[0])
        || vars.get(curr_scope).unwrap().get("vars").unwrap().get("str").unwrap().contains(&var[0])
        || vars.get("const").unwrap().get("str").unwrap().contains_key(&var[0]){
            stmt.push(var[0].clone());
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    }
    
    return stmt;

}

fn parse_ret(expr: Vec<String>, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_scope: &String) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["ret".to_string()]; 
    if vars.get("funcs").unwrap().get("int").unwrap().contains_key(curr_scope) {
        if vars.get("const").unwrap().get("int").unwrap().contains_key(&expr[1]) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot return constants");
            exit(1);
        }
        stmt = parse_int(&expr[1..], vars, &"var".to_string(), curr_scope, stmt)
    } else if vars.get("funcs").unwrap().get("str").unwrap().contains_key(curr_scope) {
        if vars.get("const").unwrap().get("str").unwrap().contains_key(&expr[1]) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot return constants");
            exit(1);
        }
        stmt = parse_str(&expr[1..], vars, &"var".to_string(), curr_scope, stmt)
    } else if vars.get("funcs").unwrap().get("bool").unwrap().contains_key(curr_scope) {
        if vars.get("const").unwrap().get("bool").unwrap().contains_key(&expr[1]) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot return constants");
            exit(1);
        }
        stmt = parse_bool(&expr[1..], vars, &"var".to_string(), curr_scope, stmt)
    } else {
        println!("unknown return type");
        exit(11);
    }
    return stmt;
}

fn parse_function_arguments(name: &String, args:&[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_scope: &String, mut stmt: Vec<String>) -> Vec<String> {
    let types: &[String] = &vars.get(name).unwrap().get("args").unwrap().get("types").unwrap().as_slice();
    let mut args_list = vec![];
    let mut buf = vec![];
    for i in 0..args.len() {
        if args[i] == "COMMA" {
            args_list.push(buf.clone());
            buf.clear();
        } else {
            buf.push(args[i].clone());
        }
    }
    if buf.len() > 0 {
        args_list.push(buf.clone());
    }
    if args_list.len() != types.len() {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected {} arguments in call to function `{}` but recieved {}", types.len(), name, args_list.len());
        exit(1);
    }
    for i in 0..args_list.len() {
        let arg = &args_list[i];
        let type_ = &types[i];
        if type_ == "int" {
            stmt = parse_int(&arg[0..], vars, &"var".to_string(), curr_scope, stmt);
        } else if type_ == "bool" {
            stmt = parse_bool(&arg[0..], vars, &"var".to_string(), curr_scope, stmt);
        }
    }
    return stmt;
}

fn parse_fnc_call(name: &String, args: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_scope: &String,mut stmt: Vec<String>) -> Vec<String> {
    stmt.push(String::from("fcall"));
    stmt.push(name.clone());
    if args == ["PARC"] {
        stmt.push(String::from(""));
    } else {
        stmt = parse_function_arguments(name, args, vars, curr_scope, stmt)
    }
    return stmt;
}

fn parse_fnc_dec(_type: &String, name: &String, args: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["fdec".to_string(), _type.to_string(), name.to_string()];
    if vars.contains_key(name) {
        eprintln!("\x1b[1mParserError\x1b[0m: cannot redeclare existing functions");
        exit(1);
    }
    let binding = args.join("");
    let mut argv = binding.split("COMMA").collect::<Vec<&str>>();
    if argv[0] == "" {
        argv.remove(0);
    }
    let mut args: Vec<String>  = vec![];
    if argv.len() > 0 && argv[0] != "0" {
        if !is_int_lit(&argv[0].to_string()) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: First item in a list of arguments must always be the count");
            exit(1);
        }
        if argv.len() == 1 {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected arguments after count declaration");
            eprintln!("For a function with 0 arguments, an argument count is not needed");
            exit(1);
        }
        for i in 1..argv.len() {
            let arg: Vec<&str> = argv[i].split("COL").collect();
            if arg.len() == 1 {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: One of `type identifier` or `function argument` is missing");
                eprintln!("Ensure that type indentifier is followed by `:`");
                exit(1);
            }
            if !TYPES.contains(&arg[0]) {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: function arguments must be preposed with a type identifier");
                exit(1);
            } else {
                args.push(argv[i].to_string());
            }
        }
        if (argv.len()-1) != argv[0].parse::<usize>().unwrap_or_default() {
            eprintln!("\x1b[1mParserError\x1b[0m: Amount of arguments specified does not equal the amount of arguments found");
            exit(1);
        }
    }
    stmt.push(args.clone().join(","));
    return stmt;
}

fn parse_var_ass(name: &String, value: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_scope: &String) -> Vec<String> {
    if vars.get("const").unwrap().get("names").unwrap().contains_key(name) {
        eprintln!("\x1b[1mParserError\x1b[0m: constant variables cannot be reassigned after creation");
        exit(1);
    }
    let mut stmt: Vec<String> = vec!["vass".to_string(), name.to_string()];
    if is_bin_expr(value) {
        stmt = parse_bin_expr(value, &"var".to_string(), stmt);
    } else {
        if vars.get(curr_scope).unwrap().get("vars").unwrap().get("int").unwrap().contains(name) 
        || vars.get(curr_scope).unwrap().get("args").unwrap().get("int").unwrap().contains(name) {
            stmt = parse_int(value,vars, &"var".to_string(), curr_scope, stmt);
        } else if vars.get(curr_scope).unwrap().get("vars").unwrap().get("str").unwrap().contains(name) 
        || vars.get(curr_scope).unwrap().get("args").unwrap().get("str").unwrap().contains(name) {
            stmt = parse_str(value,vars, &"var".to_string(), curr_scope, stmt);
        } else if vars.get(curr_scope).unwrap().get("vars").unwrap().get("bool").unwrap().contains(name) 
        || vars.get(curr_scope).unwrap().get("args").unwrap().get("bool").unwrap().contains(name){
            stmt = parse_bool(value,vars, &"var".to_string(), curr_scope, stmt);
        } else {
            println!("unkown reassignment type");
            exit(11);
        }
    }
    return stmt;
}

fn parse_var_dec(var_type: &String, attribute: &String, var: &[String], name: &String, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_scope: &String) -> Vec<String> {
    if !TYPES.to_vec().contains(&var_type.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: `{}` is an unsupported variable type.\nExpected one of `{:?}`", var_type, TYPES);
        exit(1);
    }
    if !ATTRIBUTES.to_vec().contains(&attribute.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: `{}` is an unsupported attribute.\nExpected one of `{:?}`", attribute, ATTRIBUTES);
        exit(1);
    }
    if vars.get("const").unwrap().get("names").unwrap().contains_key(name) {
        if attribute == "const" {
            eprintln!("\x1b[1mParserError\x1b[0m: cannot redeclare constant variables");
            exit(1);
        } else {

        }
    }
    if vars.get(curr_scope).unwrap().get("vars").unwrap().get("names").unwrap().contains(name) {
        eprintln!("\x1b[1mParserError\x1b[0m: variable `{name}` has already been declared in this scope")
    }

    let mut stmt: Vec<String> = vec!["vdec".to_string(), attribute.to_string(), var_type.to_string(), name.to_string()];

    if var_type == "int" {
        stmt = parse_int(var, vars, attribute,curr_scope, stmt);
    } else if var_type == "vec" {
        println!("cannot parse vec yet");
        exit(11);
    } else if var_type == "str" {
        stmt = parse_str(var, vars, attribute,curr_scope, stmt);
    } else if var_type == "bool"{
        stmt = parse_bool(var, vars, attribute,curr_scope, stmt)
    }

    return stmt;
}

fn parse_bin_expr(expr: &[String], attribute: &String, mut stmt: Vec<String>) -> Vec<String> {
    let mut expr = expr.to_vec();
    while expr.len() > 2 {
        let bin_op_loc = find_bin_operator(&expr[0..]);
        if bin_op_loc == 0 {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Binary expressions cannot begin with a binary operator");
            exit(1);
        }
        let expr_len = get_bin_expr_len(&expr);
        if attribute == "const" {
            if !is_int_lit(&expr[0..bin_op_loc].concat())
            || !is_int_lit(&expr[bin_op_loc+1..expr_len].concat()) {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use variables or functions in constant binary expressions");
                exit(1);
            }
        }
        if expr.len() == expr_len {
            if expr[bin_op_loc] == "EXP" {
                stmt = parse_bin_exp(&expr[0..expr_len], stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == "TIMES" {
                stmt = parse_bin_mult(&expr[0..expr_len], stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == "DIV" {
                stmt = parse_bin_div(&expr[0..expr_len], stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == "PLUS" {
                stmt = parse_bin_add(&expr[0..expr_len], stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == "MIN" {
                stmt = parse_bin_sub(&expr[0..expr_len], stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            }
        } else {
            let bin_op_2_loc = find_bin_operator(&expr[bin_op_loc+1..])+expr_len-1;
            if bin_op_2_loc == 0 {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Missing operand. Binary operators must be separated by an operand");
                exit(1);
            }
            let expr_2_len = get_bin_expr_len(&expr[bin_op_2_loc+1..]);
            if attribute == "const" {
                if !is_int_lit(&expr[expr_len..bin_op_2_loc].concat())
                || !is_int_lit(&expr[bin_op_2_loc+1..expr_2_len].concat()) {
                    eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use variables or functions in constant binary expressions");
                    exit(1);
                }
            }
            if expr[bin_op_loc] == "EXP" {
                stmt = parse_bin_exp(&expr[0..expr_len], stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == "TIMES" || expr[bin_op_loc] == "DIV" {
                if expr[bin_op_2_loc] == "EXP" {
                    stmt = parse_bin_exp(&expr[expr_len-1..expr_2_len+expr_len+1], stmt);
                    expr.drain(expr_len-1..expr_2_len+expr_len+1);
                    expr.insert(expr_len-1, "STACK".to_string());
                } else {
                    if expr[bin_op_loc] == "TIMES" {
                        stmt = parse_bin_mult(&expr[0..expr_len], stmt);
                        expr.drain(0..expr_len);
                        expr.insert(0, "STACK".to_string());
                    } else {
                        stmt = parse_bin_div(&expr[0..expr_len], stmt);
                        expr.drain(0..expr_len);
                        expr.insert(0, "STACK".to_string());
                    }
                }
            } else if expr[bin_op_loc] == "PLUS" || expr[bin_op_loc] == "MIN" {
                if expr.len() == expr_len+expr_2_len+1 {
                    if expr[bin_op_2_loc] == "EXP" {
                        stmt = parse_bin_exp(&expr[expr_len..expr_2_len+1], stmt);
                        expr.drain(expr_len..expr_2_len+1);
                        expr.insert(expr_len, "STACK".to_string());
                    } else if expr[bin_op_2_loc] == "TIMES" || expr[bin_op_2_loc] == "DIV" {
                        if expr[bin_op_2_loc] == "TIMES" {
                            stmt = parse_bin_mult(&expr[expr_len-1..expr_2_len+expr_len+1], stmt);
                            expr.drain(expr_len-1..expr_2_len+expr_len+1);
                            expr.insert(expr_len-1, "STACK".to_string());
                        } else {
                            stmt = parse_bin_div(&expr[expr_len-1..expr_2_len+expr_len+1], stmt);
                            expr.drain(expr_len-1..expr_2_len+expr_len+1);
                            expr.insert(expr_len-1, "STACK".to_string());
                        }
                    } else {
                        if expr[bin_op_loc] == "PLUS" {
                            stmt = parse_bin_add(&expr[0..expr_len], stmt);
                            expr.drain(0..expr_len);
                            expr.insert(0, "STACK".to_string());
                        } else {
                            stmt = parse_bin_sub(&expr[0..expr_len], stmt);
                            expr.drain(0..expr_len);
                            expr.insert(0, "STACK".to_string());
                        }
                    }
                } else {
                    let bin_op_3_loc = find_bin_operator(&expr[bin_op_2_loc+1..])+expr_2_len-1;
                    if bin_op_3_loc == 0 {
                        eprintln!("\x1b[1mSyntaxError\x1b[0m: Missing operand. Binary operators must be separated by an operand");
                        exit(1);
                    }
                    let expr_3_len = get_bin_expr_len(&expr[bin_op_3_loc+1..]);
                    if attribute == "const" {
                        if !is_int_lit(&expr[expr_2_len+1..bin_op_3_loc].concat())
                        || !is_int_lit(&expr[bin_op_3_loc+1..expr_3_len].concat()) {
                            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use variables or functions in constant binary expressions");
                            exit(1);
                        }
                    }
                    if expr[bin_op_2_loc] == "EXP" {
                        stmt = parse_bin_exp(&expr[expr_len..expr_2_len+1], stmt);
                        expr.drain(expr_len..expr_2_len+1);
                        expr.insert(expr_len, "STACK".to_string());
                    } else if expr[bin_op_2_loc] == "TIMES" || expr[bin_op_2_loc] == "DIV" {
                        if expr[bin_op_3_loc] == "EXP" {
                            stmt = parse_bin_exp(&expr[expr_2_len+1..expr_3_len+2], stmt);
                            expr.drain(expr_2_len+1..expr_3_len+2);
                            expr.insert(expr_2_len+1,"STACK".to_string());
                        } else {
                            if expr[bin_op_2_loc] == "TIMES" {
                                stmt = parse_bin_mult(&expr[expr_len..expr_2_len+1], stmt);
                                expr.drain(expr_len..expr_2_len+1);
                                expr.insert(expr_len, "STACK".to_string());
                            } else {
                                stmt = parse_bin_div(&expr[expr_len..expr_2_len+1], stmt);
                                expr.drain(expr_len..expr_2_len+1);
                                expr.insert(expr_len, "STACK".to_string());
                            }
                        }
                    } else {
                        if expr[bin_op_loc] == "PLUS" {
                            stmt = parse_bin_add(&expr[expr_len..expr_2_len+1], stmt);
                            expr.drain(expr_len..expr_2_len+1);
                            expr.insert(expr_len, "STACK".to_string());
                        } else {
                            stmt = parse_bin_sub(&expr[expr_len..expr_2_len+1], stmt);
                            expr.drain(expr_len..expr_2_len+1);
                            expr.insert(expr_len, "STACK".to_string());
                        }
                    }
                }
            }
        }
    }
    return stmt;
}

fn parse_bin_add(expr: &[String], mut stmt: Vec<String>) -> Vec<String> {
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");

    if lhs.contains("PARO") {
        lhs = format!("fcall!{}", expr[0]);
    }

    if rhs.contains("PARO") {
        rhs = format!("fcall!{}", expr[find_bin_operator(expr)+1]);
    }

    stmt.push(format!("{} PLUS {}", lhs, rhs));
    return stmt;
}

fn parse_bin_sub(expr: &[String], mut stmt: Vec<String>) -> Vec<String> {   
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");
    if lhs.contains("PARO") {
        lhs = format!("fcall!{}", expr[0]);
    }

    if rhs.contains("PARO") {
        rhs = format!("fcall!{}", expr[find_bin_operator(expr)+1]);
    }
    stmt.push(format!("{} MIN {}", lhs, rhs));
    return stmt;
}

fn parse_bin_div(expr: &[String], mut stmt: Vec<String>) -> Vec<String> {
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");

    if lhs.contains("PARO") {
        lhs = format!("fcall!{}", expr[0]);
    }

    if rhs.contains("PARO") {
        rhs = format!("fcall!{}", expr[find_bin_operator(expr)+1]);
    }

    stmt.push(format!("{} DIV {}", lhs, rhs));
    return stmt;
}

fn parse_bin_mult(expr: &[String], mut stmt: Vec<String>) -> Vec<String> {
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");

    if lhs.contains("PARO") {
        lhs = format!("fcall!{}", expr[0]);
    }

    if rhs.contains("PARO") {
        rhs = format!("fcall!{}", expr[find_bin_operator(expr)+1]);
    }

    stmt.push(format!("{} TIMES {}", lhs, rhs));
    return stmt;
}

fn parse_bin_exp(expr: &[String], mut stmt: Vec<String>) -> Vec<String> {
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");

    if lhs.contains("PARO") {
        lhs = format!("fcall!{}", expr[0]);
    }

    if rhs.contains("PARO") {
        rhs = format!("fcall!{}", expr[find_bin_operator(expr)+1]);
    }

    stmt.push(format!("{} EXP {}", lhs, rhs));
    return stmt;
}

fn parse_print(prt_type: &String, expr: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_scope: &String) -> Vec<String> {
    let mut stmt: Vec<String> = vec![]; 
    if prt_type == &"prf".to_string() 
    || prt_type == &"pre".to_string() {
        stmt.push(prt_type.to_owned());
        let mut term_buffer: Vec<String> = vec![];
        let mut print_terms: Vec<Vec<String>> = vec![];
        for token in expr {
            if token != "AMP" {
                term_buffer.push(token.clone());
            } else {
                print_terms.push(term_buffer.clone());
            }
        }
        if term_buffer.len() > 0 {
            print_terms.push(term_buffer.clone());
        }



        let mut char_buf: Vec<char> = vec![];
        let mut term_type = "var";

        for term in print_terms {
            if is_bin_expr(term.as_slice()) {
                stmt.push(parse_bin_expr(&term[0..], &"var".to_string(), vec![]).join(" "))
            } else if is_fnc_call(term.clone()) {
                stmt.push(parse_fnc_call(&term[0], &term[2..term.len()-1], vars, curr_scope,vec![]).join(" "));
            } else if term_type == "str" || term[0].chars().nth(0) == Some('"') {
                for ch in term[0].chars() {
                    if char_buf.len() == 0 {
                        if ch != '"' {
                            char_buf.push('"')
                        }
                    }
                    char_buf.push(ch);
                    if char_buf.len() == 9 {
                        if char_buf[char_buf.len()-1] != '"' {
                            char_buf.push('"');
                        } else {
                            term_type = "var";
                        }
                        stmt.push(char_buf.iter().collect());
                        char_buf.clear();
                    }
                }  
                if char_buf.len() > 0 {
                    if char_buf[char_buf.len()-1] != '"' {
                        char_buf.push('"');
                    } else {
                        term_type = "var";
                    }
                    stmt.push(char_buf.iter().collect());
                    char_buf.clear();
                }
            } else {
                stmt.push(term.join(" "));
            }
        }
    } else {
        if expr.len() > 1 {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Unformatted print statements use only one text argument.\nFor variable usage and string addition, use `prf`");
            exit(1);
        } 
        stmt.push("prt".to_string());
        stmt.push(format!("{}", &expr[0]))
    }
    return stmt;
}

fn parse_int(var: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, attribute: &String, curr_scope: &String, mut stmt:Vec<String>) -> Vec<String> {
    if var.len() > 1 {
        if is_bin_expr(var){
            stmt = parse_bin_expr(var, attribute, stmt);
        } else if var[1] == "PARO" {
            if attribute == "const" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use functions in constant variables");
                exit(1);
            }
            if vars.get("funcs").unwrap().get("str").unwrap().contains_key(&var[0]) {
                stmt = parse_fnc_call(&var[0], &var[2..var.len()-1], vars, curr_scope, stmt);
            } else {
                eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
                exit(1);
            }
        } else {
            exit(1);
        }
    } else {
        if is_int_lit(&var[0]) {
            stmt.push(var[0].to_owned()); 
        } else if vars.get(curr_scope).unwrap().get("args").unwrap().get("int").unwrap().contains(&var[0])
        || vars.get(curr_scope).unwrap().get("vars").unwrap().get("int").unwrap().contains(&var[0])
        || vars.get("const").unwrap().get("int").unwrap().contains_key(&var[0]){
            stmt.push(var[0].clone()); 
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    }

    return stmt;
}

fn parse_out(expr: Vec<String>, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_scope: &String) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["out".to_string()]; 
    if expr.len() > 2 {
        if is_bin_expr(&expr[1..]) {
            stmt = parse_bin_expr(&expr[1..],  &"out".to_string(), stmt);
        } else if is_fnc_call(expr[1..].to_vec()) {
            stmt = parse_fnc_call(&expr[1], &expr[3..expr.len()-1], vars, curr_scope, stmt);
        }
    } else {
        if is_int_lit(&expr[1]) {
            stmt.push(expr[1].clone());
        } else if vars.get(curr_scope).unwrap().get("vars").unwrap().get("int").unwrap().contains(&expr[1])
        || vars.get(curr_scope).unwrap().get("args").unwrap().get("int").unwrap().contains(&expr[1]){
            stmt.push(format!("{}", expr[1].clone()));
        } else {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Only integer literals or variables/functions evaluating to integer literals can be used as exit codes");
            exit(1);
        }
    }
    return stmt;
}


fn get_bin_expr_len(bin_expr: &[String]) -> usize {
    let lhalf = find_bin_operator(&bin_expr[0..]);
    let rhalf = find_bin_operator(&bin_expr[lhalf+1..]);
    if rhalf > 0 {
        return lhalf + rhalf+1;
    } else {
        return bin_expr.len();
    }

}

fn find_bin_operator(bin_expr: &[String]) -> usize {
    for i in 0..bin_expr.len() {
        if bin_expr[i] == "PLUS"
        || bin_expr[i] == "TIMES"
        || bin_expr[i] == "MIN"
        || bin_expr[i] == "DIV" {
            return i;
        }
    }
    return 0;
}


fn is_if(expr:Vec<String>) -> bool {
    if expr[0] != "if" {
        if expr[0] == "else" {
            if (expr.len() == 2 && expr[1] != "if") || expr.len() == 1 {
                return false;
            }
        } else {
            return false;
        }
    }
    if !expr.contains(&"EQ".to_string()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected one of `=`, `>`, `<`, `>=`, `<=`, `>>`, `<<`, `!` in conditional");
        exit(1);
    }
    if expr.len() < 4{
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Incomplete conditional statement");
        exit(1);
    } 
    return true;
}

fn is_string_lit(tok: &String) -> bool {
    if tok.chars().nth(0) == Some('"') && tok.chars().nth(tok.len()-1) == Some('"') {
        return true;
    }
    return false;
}

fn is_fnc_call(expr:Vec<String>) -> bool {
    if expr[0] == "fnc"
    || expr[0] == "out"
    || expr[0] == "ret"
    || TYPES.contains(&expr[0].as_str()) {
        return false;
    }
    if !expr.contains(&"PARO".to_string()) {
        return false;
    }
    if expr[1] != "PARO" {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `(` after function name in function call");
        exit(1);
    }
    if expr[expr.len()-1] != "PARC" {
        eprintln!("\x1b[1mSyntaxError\x1b[0m:V Expected `)` after list of function arguments");
        exit(1);
    }
    return true;
}

fn is_bin_expr(expr: &[String]) -> bool {
    if expr.contains(&"PLUS".to_string()) 
    || expr.contains(&"TIMES".to_string()) 
    || expr.contains(&"MIN".to_string())
    || expr.contains(&"DIV".to_string()) {
        return true;
    } else {
        if expr.concat().contains(&"PLUS".to_string()) 
        || expr.concat().contains(&"TIMES".to_string()) 
        || expr.concat().contains(&"MIN".to_string())
        || expr.concat().contains(&"DIV".to_string()) {
            return true;
        } else {
            return false;
        }
    }
}

fn is_ret(expr:Vec<String>) -> bool {
    if expr[0] != "ret".to_string() {
        return false;
    }
    if expr.len() < 2 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Incomplete Return Expression");
        exit(1);
    }
    return true;
}

fn is_fnc_dec(expr: Vec<String>) -> bool {
    if expr[0] != "int".to_string() && expr[0] != "str" && expr[0] != "bool" {
        return false;
    }
    if expr[1] != "fnc".to_string() {
        return false;
    }
    if expr.len() < 5 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Incomplete Function Declaration Expression");
        exit(1);
    }
    return true;
}

fn is_print(expr: Vec<String>) -> bool {
    if !matches!(expr[0].as_str(), "prt" | "prf" | "pre") {
        return false;
    }
    if expr.len() < 2 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Incomplete Print Expression");
        exit(1);
    }
    if expr[0] == "prt".to_string() {
        if expr[1].to_string().chars().nth(0) != Some('"')
        || expr[1].to_string().chars().nth(expr[1].len()-1) != Some('"'){
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Standard print statement must contain `\"` around the text");
            eprintln!("If you meant to print the value of a variable, use `prf`");
            exit(1);
        }
        if expr[1].len() == 2 {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Print statement must have text to print");
            exit(1);
        }
    }
    
    return true;
}

fn is_int_lit(tok: &String) -> bool {
    for _char in tok.chars() {
        if !_char.is_numeric() && _char != '0' {
            return false;
        }
    }
    return true;
}

fn is_out(expr: Vec<String>) -> bool {
    if expr[0] != "out".to_string() {
        return false;
    }
    if expr.len() < 2 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Incomplete Exit Expression");
        exit(1);
    }
    if expr.contains(&"PARO".to_string()) || expr.contains(&"PARC".to_string()) {
        return false;
    }
    return true;
}

fn is_var_dec(expr: Vec<String>) -> bool {
    if expr[0] != "int".to_string() && expr[0] != "str" && expr[0] != "bool" {
        return false;
    }
    if expr[1] == "fnc".to_string() {
        return false;
    }
    if expr.len() < 5 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Incomplete Variable Declaration Expression");
        exit(1);
    }
    if expr[1] != "const".to_string() && expr[1] != "var".to_string() {
        return false;
    }
    if expr[3] != "LET".to_string() {
        return false;
    }
    return true;
}

fn is_var_assignment(expr: Vec<String>) -> bool {
    if expr.len() >= 3 {
        if expr[1] != "LET".to_string() {
            return false;
        }
    } else {
        return false;
    }
    return true;
}


pub fn parse(tokens: Vec<String>) -> Vec<Vec<String>> {
    let mut has_if: bool = false;
    let mut _has_else: bool = false;
    let mut if_counter: i32 = 0;
    let mut in_if: bool = false;
    let mut vars: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = HashMap::new();
    vars.insert(
        "const".to_string(),
        HashMap::from([
            ("int".to_string(), HashMap::new()),
            ("str".to_string(), HashMap::new()),
            ("bool".to_string(), HashMap::new()),
            ("names".to_string(), HashMap::new()),
        ])
    );
    vars.insert(
        "funcs".to_string(),
        HashMap::from([
            ("int".to_string(), HashMap::new()),
            ("str".to_string(), HashMap::new()),
            ("bool".to_string(), HashMap::new()),
            ("names".to_string(), HashMap::new()),
        ])
    );
    let mut curr_func: String = String::from("");
    let mut stmts: Vec<Vec<String>> = vec![];
    let mut expr: Vec<String> = vec![];
    let mut i: usize = 0;
    while i < tokens.len() {
        let mut j = i;
        while j < tokens.len() && 
        tokens[j] != "SEMI".to_string() 
        && tokens[j] != "CURLO".to_string() 
        && tokens[j] != "CURLC".to_string() {
            expr.push(tokens[j].clone().to_string());
            j += 1;
        }   
        let last_tok = tokens[j].clone();
        if last_tok == "CURLC".to_string() {
            if !in_if {
                stmts.push(vec!["endfunc".to_string()]);
                curr_func = String::from("");
            } else {
                stmts.push(vec!["endif".to_string()]);
                if_counter -= 1;
                if if_counter == 0 {
                    in_if = false;
                }
            }
        } else if is_out(expr.clone()) {
            stmts.push(parse_out(expr.clone(), &vars, &curr_func));
        } else if is_var_dec(expr.clone()) {
            stmts.push(parse_var_dec(&expr[0], &expr[1], &expr[4..], &expr[2], &vars, &curr_func));
            if &expr[1] == "const" {
                vars.get_mut("const").unwrap().get_mut(&expr[0]).unwrap().insert(
                    expr[2].to_string(),
                    vec!["0".to_string()]
                );
                vars.get_mut("const").unwrap().get_mut("names").unwrap().insert(
                    expr[2].to_string(),
                    vec!["0".to_string()]
                );
            } else {
                vars.get_mut(&curr_func.to_string()).unwrap().get_mut("vars").unwrap().get_mut(&expr[0]).unwrap().push(expr[2].clone());
                vars.get_mut(&curr_func.to_string()).unwrap().get_mut("vars").unwrap().get_mut("names").unwrap().push(expr[2].clone());
            }
        } else if is_var_assignment(expr.clone()) {
            stmts.push(parse_var_ass(&expr[0], &expr[2..], &vars, &curr_func))
        } else if is_print(expr.clone()) {
            stmts.push(parse_print(&expr[0], &expr[1..], &vars, &curr_func));
        } else if is_fnc_dec(expr.clone()) {
            if last_tok != "CURLO" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `{{` after function declaration");
                exit(1);
            }
            if expr.len() == 5 {
                stmts.push(parse_fnc_dec(&expr[0], &expr[2], &[], &vars));
                    vars.insert(expr[2].clone(), HashMap::from([
                        ("args".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),
                            ("str".to_string(), vec![]),
                            ("bool".to_string(), vec![]),
                            ("names".to_string(), vec![]),
                            ("types".to_string(), vec![]),]
                        )),
                        ("vars".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),
                            ("str".to_string(), vec![]),
                            ("bool".to_string(), vec![]),
                            ("names".to_string(), vec![]),]
                        )),
                    ]));
                    vars.get_mut("funcs").unwrap().get_mut(&expr[0]).unwrap().insert(expr[2].clone(), vec![]);
            } else {
                stmts.push(parse_fnc_dec(&expr[0], &expr[2], &expr[4..expr.len()-1], &vars));
                    vars.insert(expr[2].clone(), HashMap::from([
                        ("args".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),
                            ("str".to_string(), vec![]),
                            ("bool".to_string(), vec![]),
                            ("names".to_string(), vec![]),
                            ("types".to_string(), vec![]),]
                        )),
                        ("vars".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),
                            ("str".to_string(), vec![]),
                            ("bool".to_string(), vec![]),
                            ("names".to_string(), vec![]),]
                        )),
                    ]));
                    vars.get_mut("funcs").unwrap().get_mut(&expr[0]).unwrap().insert(expr[2].clone(), vec![]);
                    let mut arg_type = "";
                    for arg in &expr[5..expr.len()-1] {
                        if matches!(arg.as_str(), "COL" | "COMMA") {
                            continue;
                        } else {
                            if matches!(arg.as_str(), "int" | "bool" | "str") {
                                arg_type = arg;
                                vars.get_mut(&expr[2]).unwrap().get_mut("args").unwrap().get_mut("types").unwrap().push(arg_type.to_owned());
                            } else {
                                
                                vars.get_mut(&expr[2]).unwrap().get_mut("args").unwrap().get_mut(&arg_type.to_string()).unwrap().push(arg.clone());
                            }
                        }
                    } 
            }
            curr_func = expr[2].clone();
        } else if is_ret(expr.clone()) {
            stmts.push(parse_ret(expr.clone(), &vars, &curr_func));
        } else if is_if(expr.clone()) {
            if last_tok != "CURLO" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `{{` after conditional statement");
                exit(1);
            }
            if expr[0] == "if" {
                has_if = true;
            } else {
                if !has_if {
                    eprintln!("\x1b[1mSyntaxError\x1b[0m: `else if` must follow `if`, which could not be found");
                }
            }
            if expr.contains(&"EQ".to_string()) {
                let start_point = if expr[0] == "if" {1} else {2};
                let lhs = &expr[start_point..expr.iter().position(|n| n == "EQ").unwrap()];
                let rhs = &expr[expr.iter().position(|n| n == "EQ").unwrap()+1..];
                stmts.push(parse_if(&if expr[0] == "if" {"i".to_string()} else {"ei".to_string()}, &"EQ".to_string(), lhs, rhs, &vars, &curr_func));
            }
            in_if = true;
            if_counter += 1;
            _has_else = false;
        } else if expr.len() == 1 && expr[0] == "else" {
            if !has_if {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: `else` must follow `if` or `else if`, neither of which could be found");
                exit(1);
            }
            stmts.push(vec!["else".to_string()]);
            in_if = true;
            if_counter += 1;
            _has_else = true;
        } else if expr[0] == "module"  {
            if expr.len() == 1 {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected name of module");
                exit(1);
            } else if expr.len() > 2 {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Too much information has been passed. Only include the module name after `module`");
                exit(1);
            } else {
                stmts.push(vec![expr[0].clone(), expr[1].clone()]);
            }
        } else if is_fnc_call(expr.clone()) {
            stmts.push(parse_fnc_call(&expr[0], &expr[2..expr.len()-1], &vars, &curr_func, vec![]))
        } else {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: could not parse expression `{}{}`", expr.join(" ").to_string(), last_tok);
            eprintln!("Does not match any known expression type");
            exit(1);
        } 
        expr.clear();
        i += (j-i)+1;  
    }
    return stmts;
}
