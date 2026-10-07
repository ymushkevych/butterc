use std::{collections::HashMap, process::exit};

const TYPES: [&str; 4] = ["int", "str", "bool", "null"];
const ATTRIBUTES: [&str; 2] = ["var", "const"];
const KEYWORDS: [&str; 19] = ["int", "str", "bool", "var", "const", "fnc", "ret", "out", "prt", "pre", "prf", "if", "else", "require", "from", "True", "False", "export", "module"];

fn parse_require(modules: Vec<String>, mut link_directory: String) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["require".to_string()];
    for module in modules {
        if !is_int_lit(&module) {
            stmt.push(module);
        } else {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: module in `require` cannot be an integer literal");
        }
    }
    if link_directory.chars().nth(link_directory.chars().count()-1) != Some('/') {
        link_directory.push('/');
    }
    stmt.push(link_directory);
    return stmt;
}

fn parse_export(functions: &String, vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>)) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["export".to_string()];
    let functions: Vec<String> = functions.split(',').collect::<Vec<&str>>().iter().map(|a | a.to_string()).collect();
    for function in functions { 
        if vars.1.contains_key(&function) {
            stmt.push(function.clone());
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Attempted to export function `{function}`. No such function found");
            exit(1);
        }
    }
    return stmt;
}

fn parse_if(branch: &String, condition: &String, lhs: &[String], rhs: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>) -> Vec<String> {
    if lhs.len() == 1 && KEYWORDS.contains(&lhs[0].as_str()) && !is_bool_lit(&lhs[0]) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected left-hand-side of conditional, found keyword `{}` instead", lhs[0]);
        exit(1);
    }
    if rhs.len() == 1 && KEYWORDS.contains(&rhs[0].as_str()) && !is_bool_lit(&rhs[0]) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected right-hand-side of conditional, found keyword `{}` instead", rhs[0]);
        exit(1);
    }
    let mut stmt =vec![];
    if branch == "i" {
        stmt = vec!["if".to_string()];
    } else if branch == "ei" {
        stmt = vec!["else".to_string(), "if".to_string()];
    }   
    let mut lhs_type = "";
    if is_bin_expr(lhs) && !lhs.contains(&'('.to_string()) {
        stmt = parse_bin_expr(lhs, &"var".to_string(), vars, curr_scope, stmt);
        lhs_type = "int";
    } else if lhs.len() > 1 && lhs[1] == '('.to_string() {
        if !vars.1.contains_key(&lhs[0]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}`", lhs[0]);
            exit(1);
        }
        if vars.1.get(&lhs[0]).unwrap().get("return").unwrap().contains_key(&"bool".to_string())  {
            stmt = parse_fnc_call(&lhs[0], &lhs[2..lhs.len()-1], vars, curr_scope, stmt);
            lhs_type = "bool";
        } else if vars.1.get(&lhs[0]).unwrap().get("return").unwrap().contains_key(&"int".to_string()) {
            stmt = parse_fnc_call(&lhs[0], &lhs[2..lhs.len()-1], vars, curr_scope, stmt);
            lhs_type = "int";
        }
    } else if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("int").unwrap().contains(&lhs[0])
    || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") &&  (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("int").unwrap().contains(&lhs[0]))) 
    || vars.0.get("int").unwrap().contains(&lhs[0]) || is_int_lit(&lhs[0]) {
        stmt.push(lhs[0].clone());
        lhs_type = "int";
    } else if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("bool").unwrap().contains(&lhs[0])
    || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("bool").unwrap().contains(&lhs[0]))
    || vars.0.get("bool").unwrap().contains(&lhs[0]) || is_bool_lit(&lhs[0]) {
        stmt.push(lhs[0].clone());
        lhs_type = "bool";
    } else if !(vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("all").unwrap().contains(&lhs[0])) {
        eprintln!("\x1b[1mParserError\x1b[0m: Undefined or inaccessible variable `{}` used in function `{}`", lhs[0], curr_scope[0]);
        exit(1); 
    } else {
        eprintln!("\x1b[1mParserError\x1b[0m: Unparseable boolean expression");
        exit(1);
    }
    stmt.push(condition.clone());
    if lhs_type == "int" {
        if is_bin_expr(rhs) {
            stmt = parse_bin_expr(rhs, &String::from("var"), vars, curr_scope, stmt);
        } else if rhs.len() > 1 && rhs[1] == '('.to_string() {
            if !vars.1.contains_key(&rhs[0]) {
                eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}`", rhs[0]);
                exit(1);
            }
            if vars.1.get(&lhs[0]).unwrap().get("return").unwrap().contains_key(&"int".to_string()) {
                stmt = parse_fnc_call(&rhs[0], &rhs[2..rhs.len()-1], vars.clone(), curr_scope, stmt);
            }
        } else if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("int").unwrap().contains(&rhs[0])
        || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("int").unwrap().contains(&rhs[0]) )
        || vars.0.get("int").unwrap().contains(&rhs[0]) || is_int_lit(&rhs[0]) {
            stmt.push(rhs[0].clone());
        } else if !(vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("all").unwrap().contains(&rhs[0])) {
            eprintln!("\x1b[1mParserError\x1b[0m: Undefined or inaccessible variable `{}` used in function `{}`", rhs[0], curr_scope[0]);
            exit(1);
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    } else if lhs_type == "bool" {
        if is_bin_expr(rhs) {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        } else if rhs.len() > 1 && rhs[1] == '('.to_string() {
            if !vars.1.contains_key(&rhs[0]) {
                eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}`", rhs[0]);
                exit(1);
            }
            if vars.1.get(&lhs[0]).unwrap().get("return").unwrap().contains_key(&"bool".to_string()) {
                stmt = parse_fnc_call(&rhs[0], &rhs[2..rhs.len()-1], vars, &curr_scope, stmt);
            }
        } else if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("bool").unwrap().contains(&rhs[0])
        || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") &&  vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("int").unwrap().contains(&rhs[0]))
        || vars.0.get("bool").unwrap().contains(&rhs[0]) || is_bool_lit(&rhs[0]) {
            stmt.push(rhs[0].clone());
        } else if !(vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("all").unwrap().contains(&rhs[0])) {
            eprintln!("\x1b[1mParserError\x1b[0m: Undefined or inaccessible variable `{}` used in function `{}`", rhs[0], curr_scope[0]);
            exit(1);
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    } else {
        eprintln!("\x1b[1mParserError\x1b[0m: Could not typecheck right hand side of boolean against left hand side of boolean");
        eprintln!("left hand side has no known or parseable type");
        exit(1);
    }
    return stmt;
}

fn parse_bool(var: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), attribute: &String, curr_scope: &Vec<String>, mut stmt:Vec<String>) -> Vec<String> {
    if var.len() > 1 {
        if is_binary_boolean(var) {
            if attribute == "const" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use boolean expressions when defining a constant boolean");
                exit(1);
            }
            stmt = parse_if(&"i".to_string(), &"=".to_string(), &var[0..var.iter().position(|a|a == "=").unwrap()], &var[var.iter().position(|a|a == "=").unwrap()+1..], vars.clone(), &curr_scope);
        }
        else if is_bin_expr(var) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use booleans in binary expressions");
            exit(1);
        } else if var[1] == '('.to_string() {
            if !vars.1.contains_key(&var[0]) {
                eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", var[0], curr_scope[0]);
                exit(1);
            }
            if attribute == "const" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use functions in constant variables");
                exit(1);
            }
            if vars.1.get(&var[0]).unwrap().get("return").unwrap().contains_key("bool") {
                stmt = parse_fnc_call(&var[0], &var[2..var.len()-1], vars, curr_scope, stmt);
            } else {
                eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
                exit(1);
            }
        }
    } else {
        if is_bool_lit(&var[0]) {
            stmt.push(var[0].clone());
        } else if (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("bool").unwrap().contains(&var[0]))
        || vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("bool").unwrap().contains(&var[0])
        || vars.0.get("bool").unwrap().contains(&var[0]){
            stmt.push(var[0].clone()); 
        } else if !(vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("all").unwrap().contains(&var[0])) {
            eprintln!("\x1b[1mParserError\x1b[0m: Undefined or inaccessible variable `{}` used in function `{}`", var[0], curr_scope[0]);
            exit(1);
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    }
    return stmt;
}

fn parse_str(var: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), attribute: &String, curr_scope: &Vec<String>, mut stmt:Vec<String>) -> Vec<String> {
    if var.len() > 1 {
        if is_bin_expr(var) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use strings in binary expressions");
            exit(1);
        } else if var[1] == '('.to_string() {
            if !vars.1.contains_key(&var[0]) {
                eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", var[0], curr_scope[0]);
                exit(1);
            }
            if attribute == "const" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use functions in constant variables");
                exit(1);
            }
            if vars.1.get(&var[0]).unwrap().get("return").unwrap().contains_key("str") {
                stmt = parse_fnc_call(&var[0], &var[2..var.len()-1], vars.clone(), curr_scope, stmt);
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
                    stmt.push(ch.to_string());
                }
            } else if attribute == "const" {
                stmt.push(var[0].clone());
            }
        } else if (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("str").unwrap().contains(&var[0]))
        || vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("str").unwrap().contains(&var[0])
        || vars.0.get("str").unwrap().contains(&var[0]) {
            stmt.push(var[0].clone());
        } else if !(vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("all").unwrap().contains(&var[0])) {
            eprintln!("\x1b[1mParserError\x1b[0m: Undefined or inaccessible variable `{}` used in function `{}`", var[0], curr_scope[0]);
            exit(1);
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    }
    
    return stmt;

}

fn parse_ret(expr: Vec<String>, vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["ret".to_string()]; 
    if expr.len() == 1 && KEYWORDS.contains(&expr[0].as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected value to return, found keyword `{}` instead", expr[0]);
        exit(1);
    }
    if vars.1.get(&curr_scope[0]).unwrap().get("return").unwrap().contains_key("null") {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: function `{}` is of type null, which does not allow returns", curr_scope[0]);
        eprintln!("Consider replacing `ret` with `out`");
        exit(1);
    } else if vars.1.get(&curr_scope[0]).unwrap().get("return").unwrap().contains_key("int") {
        if vars.0.get("int").unwrap().contains(&expr[1]) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot return constants");
            exit(1);
        }
        stmt = parse_int(&expr[1..], vars.clone(), &"var".to_string(), curr_scope, stmt);
    } else if vars.1.get(&curr_scope[0]).unwrap().get("return").unwrap().contains_key("str") {
        if vars.0.get("str").unwrap().contains(&expr[1]) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot return constants");
            exit(1);
        }
        stmt = parse_str(&expr[1..], vars.clone(), &"var".to_string(), curr_scope, stmt);
    } else if vars.1.get(&curr_scope[0]).unwrap().get("return").unwrap().contains_key("bool") {
        if vars.0.get("bool").unwrap().contains(&expr[1]) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot return constants");
            exit(1);
        }
        stmt = parse_bool(&expr[1..], vars.clone(), &"var".to_string(), curr_scope, stmt);
    } else {
        eprintln!("\x1b[1mParserError\x1b[0m: Undefined or inaccessible variable `{}` used in function `{}`", expr[1], curr_scope[0]);
        exit(1);
    }
    return stmt;
}

fn parse_function_arguments(name: &String, args:&[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>, mut stmt: Vec<String>) -> Vec<String> {
    let types: &[String] = &vars.1.get(name).unwrap().get("args").unwrap().get("types").unwrap().as_slice();
    let mut args_list = vec![];
    let mut buf = vec![];
    for i in 0..args.len() {
        if args[i] == "," {
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
        if arg.len() == 1 && KEYWORDS.contains(&arg[0].as_str()) && !is_bool_lit(&arg[0]) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: expected function argument, found keyword `{}` instead", arg[0]);
            exit(1);
        }
        let type_ = &types[i];
        if type_ == "int" {
            stmt = parse_int(&arg[0..], vars.clone(), &"var".to_string(), curr_scope, stmt);
        } else if type_ == "bool" {
            stmt = parse_bool(&arg[0..], vars.clone(), &"var".to_string(), curr_scope, stmt);
        } else if type_ == "str" {
            stmt = parse_str(&arg[0..], vars.clone(), &"var".to_string(), curr_scope, stmt);
        }
    }
    return stmt;
}

fn parse_fnc_call(name: &String, args: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>, mut stmt: Vec<String>) -> Vec<String> {
    stmt.push(String::from("fcall"));
    if KEYWORDS.contains(&name.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected function name, found keyword `{name}` instead");
        exit(1);
    }
    stmt.push(name.clone());
    if args == [')'.to_string()] {
        stmt.push(String::from(""));
    } else {
        stmt = parse_function_arguments(name, args, vars.clone(), curr_scope, stmt);
    }
    return stmt;
}

fn parse_fnc_dec(_type: &String, name: &String, args: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>)) -> Vec<String> {
    if KEYWORDS.contains(&name.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected function name, found keyword `{name}` instead");
        exit(1);
    }
    let mut stmt: Vec<String> = vec!["fdec".to_string(), _type.to_string(), name.to_string()];
    if vars.1.contains_key(name) {
        eprintln!("\x1b[1mParserError\x1b[0m: cannot redeclare existing functions");
        exit(1);
    }
    let binding = args.join("");
    let mut argv = binding.split(&','.to_string()).collect::<Vec<&str>>();
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
            let arg: Vec<&str> = argv[i].split(&':'.to_string()).collect();
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
    stmt.push(args.clone().join(&','.to_string()));
    return stmt;
}

fn parse_var_ass(name: &String, value: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>) -> Vec<String> {
    if vars.0.get("all").unwrap().contains(name) {
        eprintln!("\x1b[1mParserError\x1b[0m: constant variables cannot be reassigned after creation");
        exit(1);
    }
    let mut stmt: Vec<String> = vec!["vass".to_string(), name.to_string()];
    if value.len() == 1 && KEYWORDS.contains(&value[0].as_str()) && !is_bool_lit(&value[0]) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected identifier, found keyword `{}` instead", value[0]);
        exit(1);
    }
    if is_bin_expr(value) {
        stmt = parse_bin_expr(value, &"var".to_string(), vars, curr_scope, stmt);
    } else {
        if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("int").unwrap().contains(name) 
        || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("int").unwrap().contains(name)) {
            stmt = parse_int(value,vars.clone(), &"var".to_string(), curr_scope, stmt);
        } else if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("str").unwrap().contains(name) 
        || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("str").unwrap().contains(name)) {
            stmt = parse_str(value,vars.clone(), &"var".to_string(), curr_scope, stmt);
        } else if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("bool").unwrap().contains(name) 
        || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("bool").unwrap().contains(name)) {
            stmt = parse_bool(value,vars.clone(), &"var".to_string(), curr_scope, stmt);
        } else {
            eprintln!("unkown reassignment type");
            exit(11);
        }
    }
    return stmt;
}

fn parse_var_dec(var_type: &String, attribute: &String, var: &[String], name: &String, vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>) -> Vec<String> {
    if !TYPES.to_vec().contains(&var_type.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: `{}` is an unsupported variable type.\nExpected one of `{:?}`", var_type, TYPES);
        exit(1);
    }
    if !ATTRIBUTES.to_vec().contains(&attribute.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: `{}` is an unsupported attribute.\nExpected one of `{:?}`", attribute, ATTRIBUTES);
        exit(1);
    }
    if vars.0.get("all").unwrap().contains(name) {
        if attribute == "const" {
            eprintln!("\x1b[1mParserError\x1b[0m: cannot redeclare constant variables");
            exit(1);
        }
    }
    if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("all").unwrap().contains(name) {
        eprintln!("\x1b[1mParserError\x1b[0m: variable `{name}` has already been declared in this scope");
        exit(1);
    }

    let mut stmt: Vec<String> = vec!["vdec".to_string(), attribute.to_string(), var_type.to_string(), name.to_string()];
    if var.len() == 1 && KEYWORDS.contains(&var[0].as_str()) && !is_bool_lit(&var[0]) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected identifier, found keyword `{}` instead", var[0]);
        exit(1);
    }
    if var_type == "int" {
        stmt = parse_int(var, vars.clone(), attribute,curr_scope, stmt);
    } else if var_type == "str" {
        stmt = parse_str(var, vars.clone(), attribute,curr_scope, stmt);
    } else if var_type == "bool"{
        stmt = parse_bool(var, vars.clone(), attribute,curr_scope, stmt)
    }

    return stmt;
}

fn parse_bin_expr(expr: &[String], attribute: &String, vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>, mut stmt: Vec<String>) -> Vec<String> {
    let mut expr = expr.to_vec();
    while expr.len() > 2 {
        let bin_op_loc = find_bin_operator(&expr[0..]);
        if bin_op_loc == 0 {
            if expr[bin_op_loc] == "-" {

            }
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
            if expr[bin_op_loc] == '^'.to_string() {
                stmt = parse_bin_exp(&expr[0..expr_len], vars, curr_scope, stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == '*'.to_string() {
                stmt = parse_bin_mult(&expr[0..expr_len],vars, curr_scope, stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == '/'.to_string() {
                stmt = parse_bin_div(&expr[0..expr_len], vars, curr_scope,stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == '+'.to_string() {
                stmt = parse_bin_add(&expr[0..expr_len], vars, curr_scope,stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == '-'.to_string() {
                stmt = parse_bin_sub(&expr[0..expr_len], vars, curr_scope,stmt);
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
            if expr[bin_op_loc] == '^'.to_string() {
                stmt = parse_bin_exp(&expr[0..expr_len], vars, curr_scope,stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string());
            } else if expr[bin_op_loc] == '*'.to_string() || expr[bin_op_loc] == '/'.to_string() {
                if expr[bin_op_2_loc] == '^'.to_string() {
                    stmt = parse_bin_exp(&expr[expr_len-1..expr_2_len+expr_len+1], vars, curr_scope,stmt);
                    expr.drain(expr_len-1..expr_2_len+expr_len+1);
                    expr.insert(expr_len-1, "STACK".to_string());
                } else {
                    if expr[bin_op_loc] == '*'.to_string() {
                        stmt = parse_bin_mult(&expr[0..expr_len],vars, curr_scope, stmt);
                        expr.drain(0..expr_len);
                        expr.insert(0, "STACK".to_string());
                    } else {
                        stmt = parse_bin_div(&expr[0..expr_len], vars, curr_scope,stmt);
                        expr.drain(0..expr_len);
                        expr.insert(0, "STACK".to_string());
                    }
                }
            } else if expr[bin_op_loc] == '+'.to_string() || expr[bin_op_loc] == '-'.to_string() {
                if expr.len() == expr_len+expr_2_len+1 {
                    if expr[bin_op_2_loc] == '^'.to_string() {
                        stmt = parse_bin_exp(&expr[expr_len..expr_2_len+1], vars, curr_scope,stmt);
                        expr.drain(expr_len..expr_2_len+1);
                        expr.insert(expr_len, "STACK".to_string());
                    } else if expr[bin_op_2_loc] == '*'.to_string() || expr[bin_op_2_loc] == '/'.to_string() {
                        if expr[bin_op_2_loc] == '*'.to_string() {
                            stmt = parse_bin_mult(&expr[expr_len-1..expr_2_len+expr_len+1], vars, curr_scope,stmt);
                            expr.drain(expr_len-1..expr_2_len+expr_len+1);
                            expr.insert(expr_len-1, "STACK".to_string());
                        } else {
                            stmt = parse_bin_div(&expr[expr_len-1..expr_2_len+expr_len+1], vars, curr_scope,stmt);
                            expr.drain(expr_len-1..expr_2_len+expr_len+1);
                            expr.insert(expr_len-1, "STACK".to_string());
                        }
                    } else {
                        if expr[bin_op_loc] == '+'.to_string() {
                            stmt = parse_bin_add(&expr[0..expr_len], vars, curr_scope,stmt);
                            expr.drain(0..expr_len);
                            expr.insert(0, "STACK".to_string());
                        } else {
                            stmt = parse_bin_sub(&expr[0..expr_len], vars, curr_scope,stmt);
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
                    if expr[bin_op_2_loc] == '^'.to_string() {
                        stmt = parse_bin_exp(&expr[expr_len..expr_2_len+1], vars, curr_scope,stmt);
                        expr.drain(expr_len..expr_2_len+1);
                        expr.insert(expr_len, "STACK".to_string());
                    } else if expr[bin_op_2_loc] == '*'.to_string() || expr[bin_op_2_loc] == '/'.to_string() {
                        if expr[bin_op_3_loc] == '^'.to_string() {
                            stmt = parse_bin_exp(&expr[expr_2_len+1..expr_3_len+2], vars, curr_scope,stmt);
                            expr.drain(expr_2_len+1..expr_3_len+2);
                            expr.insert(expr_2_len+1,"STACK".to_string());
                        } else {
                            if expr[bin_op_2_loc] == '*'.to_string() {
                                stmt = parse_bin_mult(&expr[expr_len..expr_2_len+1], vars, curr_scope,stmt);
                                expr.drain(expr_len..expr_2_len+1);
                                expr.insert(expr_len, "STACK".to_string());
                            } else {
                                stmt = parse_bin_div(&expr[expr_len..expr_2_len+1], vars, curr_scope,stmt);
                                expr.drain(expr_len..expr_2_len+1);
                                expr.insert(expr_len, "STACK".to_string());
                            }
                        }
                    } else {
                        if expr[bin_op_loc] == '+'.to_string() {
                            stmt = parse_bin_add(&expr[expr_len..expr_2_len+1], vars, curr_scope,stmt);
                            expr.drain(expr_len..expr_2_len+1);
                            expr.insert(expr_len, "STACK".to_string());
                        } else {
                            stmt = parse_bin_sub(&expr[expr_len..expr_2_len+1],vars, curr_scope, stmt);
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

fn parse_bin_add(expr: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>, mut stmt: Vec<String>) -> Vec<String> {
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");
    if KEYWORDS.contains(&lhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected left-hand-side of addition, found keyword `{lhs}` instead");
        exit(1);
    }
    if KEYWORDS.contains(&rhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected right-hand-side of addition, found keyword `{rhs}` instead");
        exit(1);
    }
    if is_fnc_call(expr[0..find_bin_operator(expr)].to_vec()) {
        if !vars.1.contains_key(&expr[0]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[0], curr_scope[0]);
            exit(1);
        }
        let args = expr[2..find_bin_operator(expr)-1].to_vec();
        stmt = parse_fnc_call(&expr[0], &args[0..], vars, curr_scope, stmt);
        while stmt[stmt.len()-1] != "fcall" {
            stmt.pop();
        }
        stmt.pop();
        if args[0..] == [")".to_string()] {
            lhs = format!("fcall!{}$", expr[0]);
        } else {
            lhs = format!("fcall!{}${}", expr[0], args.to_vec().into_iter().filter(|a|a!=",").collect::<Vec<String>>().join("|"));          
        }
    }

    if is_fnc_call(expr[find_bin_operator(expr)+1..].to_vec()) {
        if !vars.1.contains_key(&expr[find_bin_operator(expr)+1]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[find_bin_operator(expr)+1], curr_scope[0]);
            exit(1);
        }
        let args = expr[find_bin_operator(expr)+3..expr.len()-1].to_vec();
        stmt = parse_fnc_call(&expr[find_bin_operator(expr)+1], &args[0..], vars, curr_scope, stmt);
        while stmt[stmt.len()-1] != "fcall" {
            stmt.pop();
        }
        stmt.pop();
        if args[0..] == [")".to_string()] {
            rhs = format!("fcall!{}$", expr[find_bin_operator(expr)+1]);
        } else {
            rhs = format!("fcall!{}${}", expr[find_bin_operator(expr)+1], args.to_vec().into_iter().filter(|a|a!=",").collect::<Vec<String>>().join("|"));          
        }
    }

    stmt.push(format!("{} + {}", lhs, rhs));
    return stmt;
}

fn parse_bin_sub(expr: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>, mut stmt: Vec<String>) -> Vec<String> {   
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");
    if KEYWORDS.contains(&lhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected left-hand-side of subtraction, found keyword `{lhs}` instead");
        exit(1);
    }
    if KEYWORDS.contains(&rhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected right-hand-side of subtraction, found keyword `{rhs}` instead");
        exit(1);
    }
    if is_fnc_call(expr[0..find_bin_operator(expr)].to_vec()) {
        if !vars.1.contains_key(&expr[0]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[0], curr_scope[0]);
            exit(1);
        }
        let args = expr[2..find_bin_operator(expr)-1].to_vec();
        stmt = parse_fnc_call(&expr[0], &args[0..], vars, curr_scope, stmt);
        while stmt[stmt.len()-1] != "fcall" {
            stmt.pop();
        }
        stmt.pop();
        if args[0..] == [")".to_string()] {
            lhs = format!("fcall!{}$", expr[0]);
        } else {
            lhs = format!("fcall!{}${}", expr[0], args.to_vec().into_iter().filter(|a|a!=",").collect::<Vec<String>>().join("|"));          
        }
    }

    if is_fnc_call(expr[find_bin_operator(expr)+1..].to_vec()) {
        if !vars.1.contains_key(&expr[find_bin_operator(expr)+1]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[find_bin_operator(expr)+1], curr_scope[0]);
            exit(1);
        }
        let args = expr[find_bin_operator(expr)+3..expr.len()-1].to_vec();
        stmt = parse_fnc_call(&expr[find_bin_operator(expr)+1], &args[0..], vars, curr_scope, stmt);
        while stmt[stmt.len()-1] != "fcall" {
            stmt.pop();
        }
        stmt.pop();
        if args[0..] == [")".to_string()] {
            rhs = format!("fcall!{}$", expr[find_bin_operator(expr)+1]);
        } else {
            rhs = format!("fcall!{}${}", expr[find_bin_operator(expr)+1], args.to_vec().into_iter().filter(|a|a!=",").collect::<Vec<String>>().join("|"));          
        }
    }
    stmt.push(format!("{} - {}", lhs, rhs));
    return stmt;
}

fn parse_bin_div(expr: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>, mut stmt: Vec<String>) -> Vec<String> {
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");
    if KEYWORDS.contains(&lhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected left-hand-side of division, found keyword `{lhs}` instead");
        exit(1);
    }
    if KEYWORDS.contains(&rhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected right-hand-side of division, found keyword `{rhs}` instead");
        exit(1);
    }
    if is_fnc_call(expr[0..find_bin_operator(expr)].to_vec()) {
        if !vars.1.contains_key(&expr[0]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[0], curr_scope[0]);
            exit(1);
        }
        let args = expr[2..find_bin_operator(expr)-1].to_vec();
        stmt = parse_fnc_call(&expr[0], &args[0..], vars, curr_scope, stmt);
        while stmt[stmt.len()-1] != "fcall" {
            stmt.pop();
        }
        stmt.pop();
        if args[0..] == [")".to_string()] {
            lhs = format!("fcall!{}$", expr[0]);
        } else {
            lhs = format!("fcall!{}${}", expr[0], args.to_vec().into_iter().filter(|a|a!=",").collect::<Vec<String>>().join("|"));          
        }
    }

    if is_fnc_call(expr[find_bin_operator(expr)+1..].to_vec()) {
        if !vars.1.contains_key(&expr[find_bin_operator(expr)+1]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[find_bin_operator(expr)+1], curr_scope[0]);
            exit(1);
        }
        let args = expr[find_bin_operator(expr)+3..expr.len()-1].to_vec();
        stmt = parse_fnc_call(&expr[find_bin_operator(expr)+1], &args[0..], vars, curr_scope, stmt);
        while stmt[stmt.len()-1] != "fcall" {
            stmt.pop();
        }
        stmt.pop();
        if args[0..] == [")".to_string()] {
            rhs = format!("fcall!{}$", expr[find_bin_operator(expr)+1]);
        } else {
            rhs = format!("fcall!{}${}", expr[find_bin_operator(expr)+1], args.to_vec().into_iter().filter(|a|a!=",").collect::<Vec<String>>().join("|"));          
        }
    }

    stmt.push(format!("{} / {}", lhs, rhs));
    return stmt;
}

fn parse_bin_mult(expr: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>, mut stmt: Vec<String>) -> Vec<String> {
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");
    if KEYWORDS.contains(&lhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected left-hand-side of multiplication, found keyword `{lhs}` instead");
        exit(1);
    }
    if KEYWORDS.contains(&rhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected right-hand-side of multiplication, found keyword `{rhs}` instead");
        exit(1);
    }
    if is_fnc_call(expr[0..find_bin_operator(expr)].to_vec()) {
        if !vars.1.contains_key(&expr[0]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[0], curr_scope[0]);
            exit(1);
        }
        let args = expr[2..find_bin_operator(expr)-1].to_vec();
        stmt = parse_fnc_call(&expr[0], &args[0..], vars, curr_scope, stmt);
        while stmt[stmt.len()-1] != "fcall" {
            stmt.pop();
        }
        stmt.pop();
        if args[0..] == [")".to_string()] {
            lhs = format!("fcall!{}$", expr[0]);
        } else {
            lhs = format!("fcall!{}${}", expr[0], args.to_vec().into_iter().filter(|a|a!=",").collect::<Vec<String>>().join("|"));          
        }
    }

    if is_fnc_call(expr[find_bin_operator(expr)+1..].to_vec()) {
        if !vars.1.contains_key(&expr[find_bin_operator(expr)+1]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[find_bin_operator(expr)+1], curr_scope[0]);
            exit(1);
        }
        let args = expr[find_bin_operator(expr)+3..expr.len()-1].to_vec();
        stmt = parse_fnc_call(&expr[find_bin_operator(expr)+1], &args[0..], vars, curr_scope, stmt);
        while stmt[stmt.len()-1] != "fcall" {
            stmt.pop();
        }
        stmt.pop();
        if args[0..] == [")".to_string()] {
            rhs = format!("fcall!{}$", expr[find_bin_operator(expr)+1]);
        } else {
            rhs = format!("fcall!{}${}", expr[find_bin_operator(expr)+1], args.to_vec().into_iter().filter(|a|a!=",").collect::<Vec<String>>().join("|"));          
        }
    }

    stmt.push(format!("{} * {}", lhs, rhs));
    return stmt;
}

fn parse_bin_exp(expr: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>, mut stmt: Vec<String>) -> Vec<String> {
    let mut lhs = expr[0..find_bin_operator(expr)].join("");
    let mut rhs = expr[find_bin_operator(expr)+1..].join("");
    if KEYWORDS.contains(&lhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected left-hand-side of exponentiation, found keyword `{lhs}` instead");
        exit(1);
    }
    if KEYWORDS.contains(&rhs.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected right-hand-side of exponentiation, found keyword `{rhs}` instead");
        exit(1);
    }
    if lhs.contains(&'('.to_string()) {
        if !vars.1.contains_key(&expr[0]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[0], curr_scope[0]);
            exit(1);
        }
        lhs = format!("fcall!{}$", expr[0]);
    }

    if rhs.contains(&'('.to_string()) {
        if !vars.1.contains_key(&expr[0]) {
            eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", expr[find_bin_operator(expr)+1], curr_scope[0]);
            exit(1);
        }
        rhs = format!("fcall!{}$", expr[find_bin_operator(expr)+1]);
    }

    stmt.push(format!("{} ^  {}", lhs, rhs));
    return stmt;
}

fn parse_print(prt_type: &String, expr: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>) -> Vec<String> {
    let mut stmt: Vec<String> = vec![]; 
    let mut buffer: Vec<String> = vec![];
    stmt.push(prt_type.to_owned());
    if prt_type == &"printf".to_string() 
    || prt_type == &"eprintf".to_string() {
        if expr.len() == 0 {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: `{prt_type}` must contain an item to print");
            exit(1);
        }
        for i in 0..expr.len() {
            if expr[i] != "&" {
                buffer.push(expr[i].clone())
            } else {
                if buffer.len() > 0 {
                    if is_bin_expr(&buffer[0..]) {
                        stmt = parse_bin_exp(&buffer[0..], vars, curr_scope, stmt);
                    } else if is_fnc_call(buffer.clone()) {
                        stmt = parse_fnc_call(&buffer[0], &buffer[2..buffer.len()-1], vars, curr_scope, stmt);
                        while stmt[stmt.len()-1] != "fcall" {
                            stmt.pop();
                        }
                        stmt.pop();
                        if buffer[2..buffer.len()-1] == [")"] {
                            stmt.push(format!("fcall!{}$", buffer[0]));
                        } else {
                            stmt.push(format!("fcall!{}${}", buffer[0], buffer[2..buffer.len()-1].to_vec().into_iter().filter(|a| a!=",").collect::<Vec<String>>().join("|")));
                        }
                    } else if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("all").unwrap().contains(&buffer[0])
                    || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") &&  (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("all").unwrap().contains(&buffer[0]))) 
                    || vars.0.get("all").unwrap().contains(&buffer[0])
                    || is_int_lit(&buffer[0])
                    || is_string_lit(&buffer[0]) 
                    || is_bool_lit(&buffer[0]){
                        stmt.push(buffer[0].clone());
                    } else {
                        eprintln!("\x1b[1mParserError\x1b[0m: Undefined or inaccessible variable `{}` used in function `{}`", buffer[0], curr_scope[0]);
                        exit(1);
                    }
                    buffer.clear();
                } else {
                    eprintln!("\x1b[1mSyntaxError\x1b[0m: Tried to print an empty item");
                    eprintln!("To print an empty string, print `\"\"`");
                    exit(1);
                }
            }
        }
        if buffer.len() > 0 {
            if is_bin_expr(&buffer[0..]) {
                stmt = parse_bin_exp(&buffer[0..], vars, curr_scope, stmt);
            } else if is_fnc_call(buffer.clone()) {
                stmt = parse_fnc_call(&buffer[0], &buffer[2..buffer.len()-1], vars, curr_scope, stmt);
                while stmt[stmt.len()-1] != "fcall" {
                    stmt.pop();
                }
                stmt.pop();
                if buffer[2..buffer.len()-1] == [")"] {
                    stmt.push(format!("fcall!{}$", buffer[0]));
                } else {
                    stmt.push(format!("fcall!{}${}", buffer[0], buffer[2..buffer.len()-1].to_vec().into_iter().filter(|a| a!=",").collect::<Vec<String>>().join("|")));
                }
            } else if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("all").unwrap().contains(&buffer[0])
            || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") &&  (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("all").unwrap().contains(&buffer[0]))) 
            || vars.0.get("all").unwrap().contains(&buffer[0])
            || is_int_lit(&buffer[0])
            || is_string_lit(&buffer[0]) 
            || is_bool_lit(&buffer[0]) {
                stmt.push(buffer[0].clone());
            } else {
                eprintln!("\x1b[1mParserError\x1b[0m: Undefined or inaccessible variable `{}` used in function `{}`", buffer[0], curr_scope[0]);
                exit(1);
            }
        }
        buffer.clear();
    } else {
        if expr.len() > 1 {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Unformatted print statements use only one text argument.\nFor variable usage and string addition, use `prf`");
            exit(1);
        } 
        stmt.push(format!("{}", &expr[0]))
    }
    return stmt;
}

fn parse_int(var: &[String], vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), attribute: &String, curr_scope: &Vec<String>, mut stmt:Vec<String>) -> Vec<String> {
    if var.len() > 1 {
        if is_bin_expr(var) {
            stmt = parse_bin_expr(var, attribute, vars, curr_scope, stmt);
        } else if var[1] == '('.to_string() {
            if !vars.1.contains_key(&var[0]) {
                eprintln!("\x1b[1mParserError\x1b[0m: Call to undefined function `{}` in function `{}`", var[0], curr_scope[0]);
                exit(1);
            }
            if attribute == "const" {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use functions in constant variables");
                exit(1);
            }
            if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("return").unwrap().contains_key("int") {
                stmt = parse_fnc_call(&var[0], &var[2..var.len()-1], vars, curr_scope, stmt);
            } else {
                eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch ");
                exit(1);
            }
        } else {
            exit(1);
        }
    } else {
        if (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("int").unwrap().contains(&var[0]))
        || vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("int").unwrap().contains(&var[0])
        || vars.0.get("int").unwrap().contains(&var[0]) || is_int_lit(&var[0]) {
            stmt.push(var[0].clone()); 
        } else if !(vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("all").unwrap().contains(&var[0])) {
            eprintln!("\x1b[1mParserError\x1b[0m: Undefined or inaccessible variable `{}` used in function `{}`", var[0], curr_scope[0]);
            exit(1);
        } else {
            eprintln!("\x1b[1mParserError\x1b[0m: Type Mismatch");
            exit(1);
        }
    }

    return stmt;
}

fn parse_out(expr: Vec<String>, vars: (&HashMap<String, Vec<String>>, &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>), curr_scope: &Vec<String>) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["out".to_string()]; 
    if expr.len() == 1 && KEYWORDS.contains(&expr[0].as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: expected left-hand-side of subtraction, found keyword `{}` instead", expr[0]);
        exit(1);
    }
    if expr.len() > 2 {
        if is_bin_expr(&expr[1..]) {
            stmt = parse_bin_expr(&expr[1..],  &"out".to_string(), vars, curr_scope, stmt);
        } else if is_fnc_call(expr[1..].to_vec()) {
            stmt = parse_fnc_call(&expr[1], &expr[3..expr.len()-1], vars, curr_scope, stmt);
        }
    } else {
        if is_int_lit(&expr[1]) {
            stmt.push(expr[1].clone());
        } else if vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("vars").unwrap().get("int").unwrap().contains(&expr[1])
        || (vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().contains_key("args") && vars.1.get(&curr_scope[curr_scope.len()-1]).unwrap().get("args").unwrap().get("int").unwrap().contains(&expr[1])) {
            stmt.push(format!("{}", expr[1].clone()));
        } else {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Only integer literals or variables/functions evaluating to integer literals can be used as exit codes");
            exit(1);
        }
    }
    return stmt;
}


fn is_inject(expr:Vec<String>) -> bool {
    if expr[0] != "inject" {
        return false;
    }
    if expr.len() == 1 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected code to inject");
        exit(1);
    }
    return true;
}

fn is_alias(expr:Vec<String>) -> bool {
    if expr[0] != "alias" {
        return false;
    }
    if !expr.contains(&String::from("as")) {
        return false;
    }
    if expr.iter().position(|a|a=="as").unwrap() == 1 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected term to alias, found keyword `as` instead");
        exit(1);
    } else if expr.iter().position(|a|a=="as").unwrap() == expr.len() -1 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: No alias defined after keyword `as`");
        exit(1);
    } else if expr.iter().position(|a|a=="as").unwrap() > 2 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Term to alias can be no longer than 1 token");
        exit(1);
    }
    return true;
}

fn is_require(expr:Vec<String>) -> bool {
    if !(expr[0] == "require") {
        return false;
    }
    if expr.len() == 1 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected module name after `require`");
        exit(1);
    }
    if expr[1] == "from" {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected module name, found keyword `from` instead");
        exit(1);
    }
    if expr.contains(&"from".to_string()) {
        if expr.iter().position(|f|f=="from").unwrap() == expr.len()-1 {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected module location after `from`");
            exit(1);
        }
    }
    return true;
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
        if bin_expr[i] == '+'.to_string()
        || bin_expr[i] == '*'.to_string()
        || bin_expr[i] == '-'.to_string()
        || bin_expr[i] == '/'.to_string()
        || bin_expr[i] == '^'.to_string() {
            return i;
        }
    }
    return 0;
}

fn is_binary_boolean(expr: &[String]) -> bool {
    if expr.contains(&'='.to_string())
    || expr.contains(&'>'.to_string())
    || expr.contains(&'<'.to_string())
    || expr.contains(&">=".to_string())
    || expr.contains(&"<=".to_string())
    || expr.contains(&'!'.to_string()) {
        return true;
    }
    return false;
}

fn is_export(expr:Vec<String>) -> bool {
    if expr[0] != "export" {
        return false;
    }
    if expr.len() == 1 {
        eprintln!("\x1b[1mSyntaxError\x1b[1m: Expected list of functions found after `export`");
        exit(1);
    }
    return true;
}

fn is_conditional(expr:Vec<String>) -> bool {
    if expr[0] != "if" {
        if expr[0] == "else" {
            if (expr.len() == 2 && expr[1] != "if") || expr.len() == 1 {
                return false;
            }
        } else {
            return false;
        }
    }
    if !expr.contains(&"=".to_string()) 
    && !expr.contains(&">".to_string())
    && !expr.contains(&"<".to_string())
    && !expr.contains(&">=".to_string())
    && !expr.contains(&"<=".to_string())
    && !expr.contains(&"!".to_string()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected one of `=`, `>`, `<`, `>=`, `<=`, `!` in conditional");
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

fn is_bool_lit(tok: &String) -> bool {
    if matches!(tok.as_str(), "True" | "False") {
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
    if !expr.contains(&"(".to_string()) {
        return false;
    }
    if expr[1] != "(" {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `(` after function name in function call");
        exit(1);
    }
    if expr[expr.len()-1] != ")" {
        println!("{expr:?}");
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `)` after list of function arguments");
        exit(1);
    }
    return true;
}

fn is_bin_expr(expr: &[String]) -> bool {
    if expr.contains(&'+'.to_string()) 
    || expr.contains(&'*'.to_string()) 
    || expr.contains(&'-'.to_string())
    || expr.contains(&'/'.to_string()) {
        return true;
    } else {
        if expr.concat().contains(&'+'.to_string()) 
        || expr.concat().contains(&'*'.to_string()) 
        || expr.concat().contains(&'-'.to_string())
        || expr.concat().contains(&'/'.to_string()) {
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
    if !TYPES.contains(&expr[0].as_str()) {
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
    if !matches!(expr[0].as_str(), "print" | "printf" | "eprint" | "eprintf") {
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
    if expr.contains(&"(".to_string()) || expr.contains(&")".to_string()) {
        return false;
    }
    return true;
}

fn is_var_dec(expr: Vec<String>) -> bool {
    if !TYPES.contains(&expr[0].as_str()) {
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
    if expr[3] != "->".to_string() {
        return false;
    }
    return true;
}

fn is_var_assignment(expr: Vec<String>) -> bool {
    if expr.len() >= 3 {
        if expr[1] != "->".to_string() {
            return false;
        }
    } else {
        return false;
    }
    return true;
}


pub fn get_injections(tokens: Vec<String>) -> Vec<Vec<String>> {
    let mut injections: Vec<Vec<String>> = vec![];
    let mut expr: Vec<String> = vec![];
    let mut i: usize = 0;
    while i < tokens.len() {
        let mut j = i;
        while j < tokens.len() && 
        tokens[j] != ';'.to_string() 
        && tokens[j] != '{'.to_string() 
        && tokens[j] != '}'.to_string() {
            expr.push(tokens[j].clone().to_string());
            j += 1;
        }
        let last_tok = tokens[j].clone();
        if expr.len() == 0 {expr.push(last_tok);}
        if is_inject(expr.clone()) {
            if !injections.contains(&expr[1..expr.len()].to_vec()) {
                injections.push(expr[1..expr.len()].to_vec());
                let ilen = injections.len();
                injections[ilen-1].push(String::from(";"));
            }
        }
        expr.clear();
        i+=(j-i)+1;
    }   
    return injections;
}

pub fn get_aliases(tokens: Vec<String>) -> HashMap<String, Vec<String>> {
    let mut aliases: HashMap<String, Vec<String>> = HashMap::new();
    let mut expr: Vec<String> = vec![];
    let mut i: usize = 0;
    while i < tokens.len() {
        let mut j = i;
        while j < tokens.len() && 
        tokens[j] != ';'.to_string() 
        && tokens[j] != '{'.to_string() 
        && tokens[j] != '}'.to_string() {
            expr.push(tokens[j].clone().to_string());
            j += 1;
        }
        let last_tok = tokens[j].clone();
        if expr.len() == 0 {expr.push(last_tok);}
        if is_alias(expr.clone()) {
            aliases.insert(expr[1].clone(), expr[3..expr.len()].to_vec());
        }
        expr.clear();
        i+=(j-i)+1;
    }   
    return aliases;
}

pub fn alias(mut tokens: Vec<String>, alias_rules: HashMap<String, Vec<String>>) -> Vec<String> {
    let mut i: usize = 0;
    while i < tokens.len() {
        let tok= &mut tokens[i].clone();
        if tok == "alias" {
            while tokens[i] != ";" {
                i+=1;
            }
            continue;
        }
        if alias_rules.contains_key(tok) {
            tokens.splice(i..i+alias_rules.get(tok).unwrap().len()-1, alias_rules.get(tok).unwrap().clone());
            i+=alias_rules.get(tok).unwrap().len()-1;
        } else {
            i+=1;
        }
    }
    return tokens;
}

pub fn get_imports(tokens: Vec<String>) -> Vec<String> {
    let mut extern_modules: Vec<String> = vec![];
    let mut stmts: Vec<Vec<String>> = vec![];
    let mut expr: Vec<String> = vec![];
    let mut i: usize = 0;
    while i < tokens.len() {
        let mut j = i;
        while j < tokens.len() && 
        tokens[j] != ';'.to_string() 
        && tokens[j] != '{'.to_string() 
        && tokens[j] != '}'.to_string() {
            expr.push(tokens[j].clone().to_string());
            j += 1;
        }
        let last_tok = tokens[j].clone();
        if expr.len() == 0 {expr.push(last_tok);}
        if is_require(expr.clone()) {
            if expr.contains(&"from".to_string()) {
                stmts.push(parse_require(expr[1..expr.iter().position(|f|f=="from").unwrap()].concat().split(',').collect::<Vec<&str>>().iter().map(|m|m.to_string()).collect::<Vec<String>>(), expr[expr.iter().position(|f|f=="from").unwrap()+1].to_string()));
            } else {
                stmts.push(parse_require(expr[1..].concat().split(',').collect::<Vec<&str>>().iter().map(|m|m.to_string()).collect::<Vec<String>>(), "/usr/lib/butter/".to_string()));
            }
            for k in 1..stmts[stmts.len()-1].len()-1 {
                extern_modules.push([stmts.clone()[stmts.clone().len()-1][stmts.clone()[stmts.clone().len()-1].len()-1].clone(), stmts.clone()[stmts.clone().len()-1][k].clone(), ".btr".to_string()].join("")); 
            }
        }
        expr.clear();
        i+=(j-i)+1;
    }   
    return extern_modules;
}

pub fn preprocess(tokens: Vec<String>) -> HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> {
    let mut functions: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = HashMap::new();
    let mut expr: Vec<String> = vec![];
    let mut i: usize = 0;
    while i < tokens.len() {
        let mut j = i;
        while j < tokens.len() && 
        tokens[j] != ';'.to_string() 
        && tokens[j] != '{'.to_string() 
        && tokens[j] != '}'.to_string() {
            expr.push(tokens[j].clone().to_string());
            j += 1;
        }
        let last_tok = tokens[j].clone();
        if expr.len() == 0 {expr.push(last_tok.clone());}  
        if is_fnc_dec(expr.clone()) {
            if last_tok != '{'.to_string() {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `{{` after function declaration");
                exit(1);
            }
            if expr.len() == 5 {
                functions.insert(expr[2].clone(), HashMap::from([
                    ("return".to_string(), HashMap::from([(expr[0].clone(), vec![])])),
                    ("args".to_string(), HashMap::from(
                        [("int".to_string(), vec![]),
                        ("str".to_string(), vec![]),
                        ("bool".to_string(), vec![]),
                        ("all".to_string(), vec![]),
                        ("types".to_string(), vec![])]
                    )),
                    ("vars".to_string(), HashMap::from(
                        [("int".to_string(), vec![]),
                        ("str".to_string(), vec![]),
                        ("bool".to_string(), vec![]),
                        ("all".to_string(), vec![]),]
                    )),
                ]));
            } else {
                functions.insert(expr[2].clone(), HashMap::from([
                    ("return".to_string(), HashMap::from([(expr[0].clone(), vec![])])),
                    ("args".to_string(), HashMap::from(
                        [("int".to_string(), vec![]),
                        ("str".to_string(), vec![]),
                        ("bool".to_string(), vec![]),
                        ("all".to_string(), vec![]),
                        ("types".to_string(), vec![])]
                    )),
                    ("vars".to_string(), HashMap::from(
                        [("int".to_string(), vec![]),
                        ("str".to_string(), vec![]),
                        ("bool".to_string(), vec![]),
                        ("all".to_string(), vec![]),]
                    )),
                ]));
                let mut k = 0;
                while k < expr[5..expr.len()-1].len() {
                    let arg = &expr[5..expr.len()-1][k];
                    if matches!(arg.as_str(), ":" | ",") {
                        k+=1;
                        continue;
                    } else {
                        if TYPES.contains(&arg.as_str()) {
                            k+=1;
                            continue;
                        } else {
                            functions.get_mut(&expr[2]).unwrap().get_mut("args").unwrap().get_mut(&(expr[5..expr.len()-1][k-2]).to_string()).unwrap().push(arg.clone());
                            functions.get_mut(&expr[2]).unwrap().get_mut("args").unwrap().get_mut("all").unwrap().push(arg.clone());
                            functions.get_mut(&expr[2]).unwrap().get_mut("args").unwrap().get_mut("types").unwrap().push((expr[5..expr.len()-1][k-2]).clone());
                        }
                    }
                    k+=1;
                } 
            }
        }
        expr.clear();
        i+=(j-i)+1;
    }
    return functions;
}

pub fn parse(tokens: Vec<String>, mut functions: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>) -> Vec<Vec<String>> {
    let mut constants: HashMap<String, Vec<String>> = HashMap::from([
        ("int".to_string(), vec![]),
        ("str".to_string(), vec![]),
        ("bool".to_string(), vec![]),
        ("all".to_string(), vec![]),
    ]);
    let mut scope_tracker: Vec<String> = vec![];
    let mut if_statements: HashMap<String, Vec<String>> = HashMap::from(
        [
            ("line".to_string(), vec![]),
            ("starting depth".to_string(), vec![]),
            ("jumps".to_string(), vec![]),
            ("closed ifs".to_string(), vec![]),
            ("completed ifs".to_string(), vec![]),
            ("current conditional type".to_string(), vec![]),
        ]
    );
    let mut curr_if: u32 = 0;
    let mut if_depth: u32 = 0;
    let mut is_module: bool = false;
    let mut stmts: Vec<Vec<String>> = vec![];
    let mut expr: Vec<String> = vec![];

    let mut i: usize = 0;
    while i < tokens.len() {
        let mut j = i;
        while j < tokens.len() && 
        tokens[j] != ';'.to_string() 
        && tokens[j] != '{'.to_string() 
        && tokens[j] != '}'.to_string() {
            expr.push(tokens[j].clone().to_string());
            j += 1;
        }   
        let last_tok = tokens[j].clone();
        if expr.len() == 0 {expr.push(last_tok.clone());}
        if last_tok == '}'.to_string() {
            if if_depth == 0 {
                stmts.push(vec!["endfunc".to_string()]);
            } else {
                if_statements.get_mut(&"closed ifs".to_string()).unwrap().push(curr_if.to_string());
                stmts.push(vec!["endif".to_string()]);
                if tokens[i+(j-i)+1] != "else" {
                    for t in 0..if_statements.get(&"line".to_string()).unwrap().len() {
                        let line = &if_statements.clone().get(&"line".to_string()).unwrap().clone()[t];
                        if !if_statements.clone().get(&"completed ifs".to_string()).unwrap().contains(&line.clone().to_string()) && 
                        if_depth == if_statements.get(&"starting depth".to_string()).unwrap()[t].clone().parse::<u32>().unwrap() {
                            stmts[if_statements.get(&"line".to_string()).unwrap()[t].clone().parse::<usize>().unwrap()].insert(
                            if if_statements.get(&"current conditional type".to_string()).unwrap()[t] == "else if" {
                                    2
                                } else {
                                    1
                                }, 
                        if tokens[i+(j-i)+1] != '}'.to_string() {
                                    (if_statements.get(&"jumps".to_string()).unwrap()[t].parse::<u32>().unwrap() + 1).to_string()
                                } else {
                                    if_statements.get(&"jumps".to_string()).unwrap()[t].parse::<u32>().unwrap().to_string()
                                });
                            stmts[if_statements.get(&"line".to_string()).unwrap()[t].clone().parse::<usize>().unwrap()].insert(
                            if if_statements.get(&"current conditional type".to_string()).unwrap()[t] == "else if" {
                                    3
                                } else {
                                    2
                                }, 
                            if tokens[i+(j-i)+1] != '}'.to_string() {
                                    "yes".to_string()
                                } else {
                                    "no".to_string()
                                } 
                            );
                            if_statements.get_mut(&"completed ifs".to_string()).unwrap().push(line.clone());
                        }
                    }
                } else {
                    for t in 0..if_statements.get_mut(&"line".to_string()).unwrap().len() {
                        if if_depth == if_statements.get_mut(&"starting depth".to_string()).unwrap()[t].clone().parse::<u32>().unwrap() {
                            if !if_statements.get_mut(&"completed ifs".to_string()).unwrap().contains(&t.to_string()) {
                                if_statements.get_mut(&"jumps".to_string()).unwrap()[t] 
                                = (if_statements.get_mut(&"jumps".to_string()).unwrap()[t].parse::<u32>().unwrap() + 1).to_string();
                            }
                        }
                    }
                }
                if_depth -= 1;
            }
            scope_tracker.pop();
        } else if is_alias(expr.clone()) || is_inject(expr.clone()) {
        } else if is_out(expr.clone()) {
            stmts.push(parse_out(expr.clone(), (&constants, &functions), &scope_tracker));
        } else if is_var_dec(expr.clone()) {
            stmts.push(parse_var_dec(&expr[0], &expr[1], &expr[4..], &expr[2], (&constants, &functions), &scope_tracker));
            if &expr[1] == "const" {
                constants.get_mut(&expr[0]).unwrap().push(expr[2].clone());
                constants.get_mut(&"all".to_string()).unwrap().push(expr[2].clone());
            } else {
                functions.get_mut(&scope_tracker[scope_tracker.len()-1].to_string()).unwrap().get_mut("vars").unwrap().get_mut(&expr[0]).unwrap().push(expr[2].clone());
                functions.get_mut(&scope_tracker[scope_tracker.len()-1].to_string()).unwrap().get_mut("vars").unwrap().get_mut("all").unwrap().push(expr[2].clone());
            }
        } else if is_var_assignment(expr.clone()) {
            stmts.push(parse_var_ass(&expr[0], &expr[2..], (&constants, &functions), &scope_tracker));
        } else if is_print(expr.clone()) {
            stmts.push(parse_print(&expr[0], &expr[1..], (&constants, &functions), &scope_tracker));
        } else if is_fnc_dec(expr.clone()) {
            if last_tok != '{'.to_string() {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `{{` after function declaration");
                exit(1);
            }
            if expr.len() == 5 {
                stmts.push(parse_fnc_dec(&expr[0], &expr[2], &[], (&constants, &functions)));
                    functions.insert(expr[2].clone(), HashMap::from([
                        ("return".to_string(), HashMap::from([(expr[0].clone(), vec![])])),
                        ("args".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),
                            ("str".to_string(), vec![]),
                            ("bool".to_string(), vec![]),
                            ("all".to_string(), vec![]),
                            ("types".to_string(), vec![])]
                        )),
                        ("vars".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),
                            ("str".to_string(), vec![]),
                            ("bool".to_string(), vec![]),
                            ("all".to_string(), vec![]),]
                        )),
                    ]));
            } else {
                stmts.push(parse_fnc_dec(&expr[0], &expr[2], &expr[4..expr.len()-1], (&constants, &functions)));
                    functions.insert(expr[2].clone(), HashMap::from([
                        ("return".to_string(), HashMap::from([(expr[0].clone(), vec![])])),
                        ("args".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),
                            ("str".to_string(), vec![]),
                            ("bool".to_string(), vec![]),
                            ("all".to_string(), vec![]),
                            ("types".to_string(), vec![])]
                        )),
                        ("vars".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),
                            ("str".to_string(), vec![]),
                            ("bool".to_string(), vec![]),
                            ("all".to_string(), vec![]),]
                        )),
                    ]));
                    let mut k = 0;
                    while k < expr[5..expr.len()-1].len() {
                        let arg = &expr[5..expr.len()-1][k];
                        if matches!(arg.as_str(), ":" | ",") {
                            k+=1;
                            continue;
                        } else {
                            if TYPES.contains(&arg.as_str()) {
                                k+=1;
                                continue;
                            } else {
                                functions.get_mut(&expr[2]).unwrap().get_mut("args").unwrap().get_mut(&(expr[5..expr.len()-1][k-2]).to_string()).unwrap().push(arg.clone());
                                functions.get_mut(&expr[2]).unwrap().get_mut("args").unwrap().get_mut("all").unwrap().push(arg.clone());
                                functions.get_mut(&expr[2]).unwrap().get_mut("args").unwrap().get_mut("types").unwrap().push((expr[5..expr.len()-1][k-2]).clone());
                            }
                        }
                        k+=1;
                    } 
            }
            scope_tracker.push(expr[2].clone());
        } else if is_ret(expr.clone()) {
            stmts.push(parse_ret(expr.clone(), (&constants, &functions), &scope_tracker));
        } else if is_conditional(expr.clone()) {
            if last_tok != '{'.to_string() {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `{{` after conditional statement");
                exit(1);
            } 
            let mut start_point = 1;
            if expr[0] == "if" {
                if_statements.get_mut(&"current conditional type".to_string()).unwrap().push("if".to_string());
            } else {
                if stmts[stmts.len()-1] != vec!["endif"] {
                    eprintln!("\x1b[1mSyntaxError\x1b[0m: `else if` must follow `if` or another `else if`");
                    eprintln!("`else if` cannot start an if-stack. Use `if` instead");
                    exit(1);
                }
                if_statements.get_mut(&"current conditional type".to_string()).unwrap().push("else if".to_string());
                start_point += 1;
            }
            curr_if += 1;   
            if_statements.get_mut(&"line".to_string()).unwrap().push(stmts.len().to_string());
            if_statements.get_mut(&"starting depth".to_string()).unwrap().push((if_depth+1).to_string());
            if_statements.get_mut(&"jumps".to_string()).unwrap().push(0.to_string());
            if_depth += 1;
            if expr.contains(&"=".to_string()) {
                let lhs = &expr[start_point..expr.iter().position(|n| n == "=").unwrap()];
                let rhs = &expr[expr.iter().position(|n| n == "=").unwrap()+1..];
                stmts.push(parse_if(&if expr[0] == "if" {"i".to_string()} else {"ei".to_string()}, &"=".to_string(), lhs, rhs, (&constants, &functions), &scope_tracker));
            } else if expr.contains(&">".to_string()) {
                let lhs = &expr[start_point..expr.iter().position(|n| n == ">").unwrap()];
                let rhs = &expr[expr.iter().position(|n| n == ">").unwrap()+1..];
                stmts.push(parse_if(&if expr[0] == "if" {"i".to_string()} else {"ei".to_string()}, &">".to_string(), lhs, rhs, (&constants, &functions), &scope_tracker));
            } else if expr.contains(&"<".to_string()) {
                let lhs = &expr[start_point..expr.iter().position(|n| n == "<").unwrap()];
                let rhs = &expr[expr.iter().position(|n| n == "<").unwrap()+1..];
                stmts.push(parse_if(&if expr[0] == "if" {"i".to_string()} else {"ei".to_string()}, &"<".to_string(), lhs, rhs, (&constants, &functions), &scope_tracker));
            } else if expr.contains(&">=".to_string()) {
                let lhs = &expr[start_point..expr.iter().position(|n| n == ">=").unwrap()];
                let rhs = &expr[expr.iter().position(|n| n == ">=").unwrap()+1..];
                stmts.push(parse_if(&if expr[0] == "if" {"i".to_string()} else {"ei".to_string()}, &">=".to_string(), lhs, rhs, (&constants, &functions), &scope_tracker));
            } else if expr.contains(&"<=".to_string()) {
                let lhs = &expr[start_point..expr.iter().position(|n| n == "<=").unwrap()];
                let rhs = &expr[expr.iter().position(|n| n == "<=").unwrap()+1..];
                stmts.push(parse_if(&if expr[0] == "if" {"i".to_string()} else {"ei".to_string()}, &"<=".to_string(), lhs, rhs, (&constants, &functions), &scope_tracker));
            } else if expr.contains(&"!".to_string()) {
                let lhs = &expr[start_point..expr.iter().position(|n| n == "!").unwrap()];
                let rhs = &expr[expr.iter().position(|n| n == "!").unwrap()+1..];
                stmts.push(parse_if(&if expr[0] == "if" {"i".to_string()} else {"ei".to_string()}, &"!".to_string(), lhs, rhs, (&constants, &functions), &scope_tracker));
            }
            scope_tracker.push(curr_if.clone().to_string());
            functions.insert(curr_if.clone().to_string(), HashMap::from([
                ("return".to_string(), HashMap::from([("null".to_string(), vec![])])),
                ("vars".to_string(), HashMap::from(
                    [("int".to_string(), vec![]),
                    ("str".to_string(), vec![]),
                    ("bool".to_string(), vec![]),
                    ("all".to_string(), vec![]),]
                )),
            ]));
            if scope_tracker.len() > 1 {
                if scope_tracker[scope_tracker.len()-2].parse::<i32>().is_ok() {
                    for integer in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("int").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("int").unwrap().push(integer.clone());
                    }
                    for string in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("str").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("str").unwrap().push(string.clone());
                    }
                    for boolean in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("bool").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("bool").unwrap().push(boolean.clone());
                    }
                    for all in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("all").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("all").unwrap().push(all.clone());
                    }
                } else {
                    for integer_arg in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("args").unwrap().get("int").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("int").unwrap().push(integer_arg.clone());
                    }
                    for string_arg in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("args").unwrap().get("str").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("str").unwrap().push(string_arg.clone());
                    }
                    for boolean_arg in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("args").unwrap().get("bool").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("bool").unwrap().push(boolean_arg.clone());
                    }
                    for all_arg in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("args").unwrap().get("all").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("all").unwrap().push(all_arg.clone());
                    }

                    for integer in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("int").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("int").unwrap().push(integer.clone());
                    }
                    for string in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("str").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("str").unwrap().push(string.clone());
                    }
                    for boolean in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("bool").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("bool").unwrap().push(boolean.clone());
                    }
                    for all in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("all").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("all").unwrap().push(all.clone());
                    }                    
                }
            }
        } else if &expr[0] == "module"  {
            if expr.len() == 1 {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected name of module");
                exit(1);
            } else if expr.len() > 2 {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Too much information has been passed. Only include the module name after `module`");
                exit(1);
            } else {
                stmts.push(vec![expr[0].clone(), expr[1].clone()]);
            }
            is_module = true;
        } else if is_fnc_call(expr.clone()) {
            stmts.push(parse_fnc_call(&expr[0], &expr[2..expr.len()-1], (&constants, &functions), &scope_tracker, vec![]));
        } else if &expr[0] == "else" {
            if stmts[stmts.len()-1] != vec!["endif"] {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: `else` must exist after `if` or `else if`");
                eprintln!("`else` cannot start an if-stack. Use `if` instead");
                exit(1);
            } else {
                if expr.len() > 1 {
                    eprintln!("\x1b[1mSyntaxError\x1b[0m: `else` cannot have arguments. Use `else if` if you want arguments");
                    exit(1);
                }
                let mut parent_if: u32 = curr_if;
                for i in (0..if_statements.get(&"starting depth".to_string()).unwrap().len()).rev() {
                    let depth = &if_statements.get(&"starting depth".to_string()).unwrap()[i];
                    if depth.parse::<u32>().unwrap() == if_depth+1 {
                        parent_if = i as u32;
                    }
                }
                if if_statements.get_mut(&"current conditional type".to_string()).unwrap().len() == 0 || if_statements.get_mut(&"current conditional type".to_string()).unwrap()[parent_if as usize] == "else" {
                    eprintln!("\x1b[1mSyntaxError\x1b[0m: `else` cannot follow another `else`");
                    eprintln!("`else` can only end an if-stack. `else` cannot exist intermediately within an if-stack");
                    exit(1);
                }
                if_statements.get_mut(&"line".to_string()).unwrap().push(stmts.len().to_string());
                if_statements.get_mut(&"starting depth".to_string()).unwrap().push((if_depth+1).to_string());
                if_statements.get_mut(&"jumps".to_string()).unwrap().push(0.to_string());
                if_statements.get_mut(&"current conditional type".to_string()).unwrap().push("else".to_string());
                stmts.push(vec!["else".to_string()]);
            }
            curr_if+=1;
            if_depth +=1;
            scope_tracker.push(curr_if.clone().to_string());
            functions.insert(curr_if.clone().to_string(), HashMap::from([
                ("return".to_string(), HashMap::from([("null".to_string(), vec![])])),
                ("vars".to_string(), HashMap::from(
                    [("int".to_string(), vec![]),
                    ("str".to_string(), vec![]),
                    ("bool".to_string(), vec![]),
                    ("all".to_string(), vec![]),]
                )),
            ]));
            if scope_tracker.len() > 1 {
                if scope_tracker[scope_tracker.len()-2].parse::<i32>().is_ok() {
                    for integer in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("int").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("int").unwrap().push(integer.clone());
                    }
                    for string in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("str").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("str").unwrap().push(string.clone());
                    }
                    for boolean in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("bool").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("bool").unwrap().push(boolean.clone());
                    }
                    for all in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("all").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("all").unwrap().push(all.clone());
                    }
                } else {
                    for integer_arg in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("args").unwrap().get("int").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("int").unwrap().push(integer_arg.clone());
                    }
                    for string_arg in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("args").unwrap().get("str").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("str").unwrap().push(string_arg.clone());
                    }
                    for boolean_arg in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("args").unwrap().get("bool").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("bool").unwrap().push(boolean_arg.clone());
                    }
                    for all_arg in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("args").unwrap().get("all").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("all").unwrap().push(all_arg.clone());
                    }

                    for integer in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("int").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("int").unwrap().push(integer.clone());
                    }
                    for string in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("str").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("str").unwrap().push(string.clone());
                    }
                    for boolean in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("bool").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("bool").unwrap().push(boolean.clone());
                    }
                    for all in functions.clone().get(&scope_tracker[scope_tracker.len()-2]).unwrap().get("vars").unwrap().get("all").unwrap() {
                        functions.get_mut(&scope_tracker[scope_tracker.len()-1]).unwrap().get_mut("vars").unwrap().get_mut("all").unwrap().push(all.clone());
                    }                    
                }
            }
        } else if is_export(expr.clone()) {
            if is_module {
                stmts.push(parse_export(&expr[1..].concat(), (&constants, &functions)));
            } else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Cannot use `export` on non-module files");
                exit(1);
            }
        } else if is_require(expr.clone()) {
            if expr.contains(&"from".to_string()) {
                stmts.push(parse_require(expr[1..expr.iter().position(|f|f=="from").unwrap()].concat().split(',').collect::<Vec<&str>>().iter().map(|m|m.to_string()).collect::<Vec<String>>(), expr[expr.iter().position(|f|f=="from").unwrap()+1].to_string()));
            } else {
                stmts.push(parse_require(expr[1..].concat().split(',').collect::<Vec<&str>>().iter().map(|m|m.to_string()).collect::<Vec<String>>(), "/usr/lib/butter/".to_string()));
            }
        } else {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: could not parse expression `{}{}`", expr.join(&' '.to_string()).to_string(), last_tok);
            eprintln!("Does not match any known expression type");
            eprintln!("Ensure that you are not missing any semicolons");
            exit(1);
        } 
        expr.clear();
        i += (j-i)+1;  
    }
    return stmts;
}
