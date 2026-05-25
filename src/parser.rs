use std::{process::exit, collections::HashMap};

const TYPES: [&str; 4] = ["int", "vec", "str", "bool"];
const ATTRIBUTES: [&str; 2] = ["var", "const"];

fn parse_ret(expr: Vec<String>, vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, curr_scope: &String) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["ret".to_string()]; 
    if expr.len() > 2 {
        if is_bin_expr(&expr[1..]) {
            stmt = parse_bin_expr(&expr[1..], stmt);
        } else {
            if expr[2] == "PARO" {
                stmt = parse_inline_fnc_call(&expr[1], &expr[3..expr.len()-1],stmt);
            } else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected Parentheses for Function Arguments");
                exit(1);
            }
        }
    } else {
        if is_int_lit(&expr[1]) {
            stmt.push(expr[1].clone());
        } else if vars.get(curr_scope).unwrap().get("vars").unwrap().get("int").unwrap().contains(&expr[1]) 
        || vars.get(curr_scope).unwrap().get("args").unwrap().get("int").unwrap().contains(&expr[1]) {
            stmt.push(format!("{}", expr[1].clone()));
        } else {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Only integer literals or variables evaluating to integer literals can be returned");
            exit(1);
        }
    }
    return stmt;
}

fn parse_inline_fnc_call(name: &String, args: &[String], mut stmt: Vec<String>) -> Vec<String> {
    let mut func: Vec<String> = vec!["fcall".to_string(), name.to_string()];
    if args == ["PARC"] {
        func.push("0".to_string());
    } else {
        func.push(args.concat().split("COMMA").collect::<Vec<&str>>().join(","));
    }
    stmt.push(func.join(" "));
    return stmt;
}

fn parse_fnc_call(name: &String, args: &[String]) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["fcall".to_string(), name.to_string()];
    if args == ["PARC"] {
        stmt.push("0".to_string());
    } else {
        stmt.push(args.concat().split("COMMA").collect::<Vec<&str>>().join(","));
    }

    return stmt;
}

fn parse_fnc_dec(name: &String, args: &[String], vars: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["fdec".to_string(), name.to_string()];
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

fn parse_var_ass(name: &String, value: &[String], consts: &Vec<String>) -> Vec<String> {
    if is_const_var(name, consts) {
        eprintln!("\x1b[1mParserError\x1b[0m: Cannot reassign value to constant variable");
        exit(1);
    }
    let mut stmt: Vec<String> = vec!["vass".to_string(), name.to_string()];
    if value.len() > 1 {
        stmt = parse_bin_expr(value, stmt);
    } else {
        if !is_int_lit(&value[0]) {
            exit(11);
        }
        stmt.push(value[0].to_string());
    }
    return stmt;
}

fn parse_var_dec(var_type: &String, attribute: &String, var: &[String], name: &String, consts: &Vec<String>) -> Vec<String> {
    if is_const_var(name, consts) {
        eprintln!("\x1b[1mParserError\x1b[0m: Cannot redeclare constant variable");
        exit(1);
    }
    if !TYPES.to_vec().contains(&var_type.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: `{}` is an unsupported variable type.\nExpected one of `{:?}`", var_type, TYPES);
        exit(1);
    }
    if !ATTRIBUTES.to_vec().contains(&attribute.as_str()) {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: `{}` is an unsupported attribute.\nExpected one of `{:?}`", attribute, ATTRIBUTES);
        exit(1);
    }

    let mut stmt: Vec<String> = vec!["vdec".to_string(), attribute.to_string(), name.to_string()];

    if var_type == "int" {
        stmt = parse_int(var, stmt);
    } else if var_type == "vec" {
        exit(11);
    } else if var_type == "str" {
        exit(11);
    } else if var_type == "bool"{
        exit(11);
    }

    return stmt;
}

fn parse_bin_expr(expr: &[String], mut stmt: Vec<String>) -> Vec<String> {
    let mut expr = expr.to_vec();
    while expr.len() > 2 {
        let bin_op_loc = find_bin_operator(&expr[0..]);
        if bin_op_loc == 0 {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Binary expressions cannot begin with a binary operator");
            exit(1);
        }
        let expr_len = get_bin_expr_len(&expr);

        if expr[bin_op_loc] == "PLUS" || expr[bin_op_loc] == "MIN" {
            if expr.len() == expr_len {
                if expr[bin_op_loc] == "PLUS" {
                    stmt = parse_bin_add(&expr[0..expr_len], stmt);
                    expr.drain(0..expr_len);
                    expr.insert(0, "STACK".to_string())
                } else {
                    stmt = parse_bin_sub(&expr[0..expr_len], stmt);
                    expr.drain(0..expr_len);
                    expr.insert(0, "STACK".to_string())
                }
            } else {
                let bin_op_2_loc = find_bin_operator(&expr[bin_op_loc+1..]);
                if bin_op_2_loc == 0 {
                    eprintln!("\x1b[1mSyntaxError\x1b[0m: Missing operand. Binary operators must be separated by an operand");
                    exit(1);
                }
                let expr_2_len = get_bin_expr_len(&expr[bin_op_2_loc+1..]);
                if expr[bin_op_2_loc] == "TIMES" || expr[bin_op_2_loc] == "DIV" {
                    if expr[bin_op_2_loc] == "TIMES" {
                        stmt = parse_bin_mult(&expr[expr_len..expr_2_len+1], stmt);
                        expr.drain(expr_len..expr_2_len+1);
                        expr.insert(expr_len, "STACK".to_string())
                    } else {
                        stmt = parse_bin_div(&expr[expr_len..expr_2_len+1], stmt);
                        expr.drain(expr_len..expr_2_len+1);
                        expr.insert(expr_len, "STACK".to_string())
                    }
                } else if expr[bin_op_2_loc] == "PLUS" || expr[bin_op_2_loc] == "MIN"{
                    if expr[bin_op_loc] == "TIMES" {
                        stmt = parse_bin_mult(&expr[0..expr_len+1], stmt);
                        expr.drain(0..expr_len);
                        expr.insert(0, "STACK".to_string())
                    } else {
                        stmt = parse_bin_div(&expr[0..expr_len+1], stmt);
                        expr.drain(0..expr_len);
                        expr.insert(0, "STACK".to_string())
                    }
                } else {
                    println!("\x1b[1mSyntaxError\x1b[0m: Used undefined or illegal binary operator");
                    exit(1);
                }
            }
        } else if expr[bin_op_loc] == "TIMES" || expr[bin_op_loc] == "DIV" {
            if expr[bin_op_loc] == "TIMES" {
                stmt = parse_bin_mult(&expr[0..expr_len], stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string())
            } else {
                stmt = parse_bin_div(&expr[0..expr_len], stmt);
                expr.drain(0..expr_len);
                expr.insert(0, "STACK".to_string())
            }
        } else {
            println!("\x1b[1mSyntaxError\x1b[0m: Used undefined or illegal binary operator");
            exit(1);
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

fn parse_print(prt_type: &String, expr: &[String]) -> Vec<String> {
    let mut stmt: Vec<String> = vec![]; 
    if prt_type == &"prf".to_string() {
        stmt.push("prf".to_string());
        let mut terms: Vec<Vec<String>> = vec![];
        let mut s_buf: Vec<String> = vec![];
        for i in 0..expr.len() {
            if expr[i] != "AMP" {
                s_buf.push(expr[i].clone());
            } else {
                terms.push(s_buf.clone());
                s_buf.clear();
            }
        }
        if s_buf.len() > 0 {
            terms.push(s_buf.clone());
            s_buf.clear();
        }
        let mut buf: Vec<char> = vec![];
        let mut _type = "var";
        for substring in terms {
            if is_bin_expr(&substring[0..]) {
                stmt = parse_bin_expr(&substring[0..],  stmt);
            } else if is_fnc_call(substring.clone()) {
                stmt = parse_inline_fnc_call(&substring[0], &substring[2..substring.len()-1], stmt);
            } else {
                let substring = &substring[0];
                if substring.chars().nth(0) == Some('"') || _type == "str" {
                    _type = "str";
                    //include quotation marks to avoid confusion with variable names. 
                    for ch in substring.chars() {
                        if buf.len() == 0 {
                            if ch != '"' {
                                buf.push('"');
                            }
                        }
                        buf.push(ch);
                        if buf.len() == 9 {
                            if buf[buf.len()-1] != '"' {
                                buf.push('"');
                            } else {
                                _type = "var";
                            }
                            stmt.push(buf.iter().collect());
                            buf.clear();
                        }
                    }
                    if buf.len() > 0 {
                        if buf[buf.len()-1] != '"' {
                            buf.push('"');
                        } else {
                            _type = "var";
                        }
                        stmt.push(buf.iter().collect());
                        buf.clear();
                    }
                } else {
                    stmt.push(substring.to_string());
                }
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

fn parse_int(var: &[String], mut stmt:Vec<String>) -> Vec<String> {
    if var.len() > 1 {
        if is_bin_expr(var){
            stmt = parse_bin_expr(var, stmt);
        } else if var[1] == "PARO" {
            stmt = parse_inline_fnc_call(&var[0], &var[2..var.len()-1], stmt);
        } else {
            exit(1);
        }
    } else {
        if is_int_lit(&var[0]) {
            stmt.push(var[0].to_owned()); 
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
            stmt = parse_bin_expr(&expr[1..],  stmt);
        } else if is_fnc_call(expr[1..].to_vec()) {
            stmt = parse_inline_fnc_call(&expr[1], &expr[3..expr.len()-1], stmt);
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
        println!("{:?}", expr);
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
    if expr[0] != "fnc".to_string() {
        return false;
    }
    if expr.len() < 4 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Incomplete Function Declaration Expression");
        exit(1);
    }
    return true;
}

fn is_const_var(tok: &String, consts: &Vec<String>) -> bool {
    return consts.contains(tok);
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
    if expr[0] != "int".to_string() && expr[0] != "vec" {
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


pub fn parse(tokens: Vec<String>, check_for_out: bool) -> Vec<Vec<String>> {
    if check_for_out {
        if !tokens.contains(&"out".to_string()) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: No exit point specified\nExit points can be specified with `out \x1b[3mn\x1b[0m;`\nwhere \x1b[3mn\x1b[0m is an integer");
            exit(1);
        }
    }
    let mut vars: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = HashMap::new();
    let mut curr_func: String = String::from("");
    let mut consts: Vec<String> = vec![];
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
        if tokens[j] == "CURLC".to_string() {
            stmts.push(vec!["endfunc".to_string()]);
            curr_func = String::from("");
        } else if is_out(expr.clone()) {
            stmts.push(parse_out(expr.clone(), &vars, &curr_func));
        } else if is_var_dec(expr.clone()) {
            stmts.push(parse_var_dec(&expr[0], &expr[1], &expr[4..], &expr[2], &consts));
            if is_const_var(&expr[2], &consts) {
                consts.push(expr[2].clone());
            } 
            if &expr[0] == "int" {
                vars.get_mut(&curr_func.to_string()).unwrap().get_mut("vars").unwrap().get_mut("int").unwrap().push(expr[2].clone());
            }
        } else if is_var_assignment(expr.clone()) {
            stmts.push(parse_var_ass(&expr[0], &expr[2..], &consts))
        } else if is_print(expr.clone()) {
            stmts.push(parse_print(&expr[0], &expr[1..]));
        } else if is_fnc_dec(expr.clone()) {
            if expr.len() == 4 {
                stmts.push(parse_fnc_dec(&expr[1], &[], &vars));
                    vars.insert(expr[1].clone(), HashMap::from([
                        ("args".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),]
                        )),
                        ("vars".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),]
                        )),
                    ]));
            } else {
                stmts.push(parse_fnc_dec(&expr[1], &expr[3..expr.len()-1], &vars));
                    vars.insert(expr[1].clone(), HashMap::from([
                        ("args".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),]
                        )),
                        ("vars".to_string(), HashMap::from(
                            [("int".to_string(), vec![]),]
                        )),
                    ]));
                    for arg in &expr[3..expr.len()-1] {
                        if !is_int_lit(arg) {
                            vars.get_mut(&expr[1]).unwrap().get_mut("args").unwrap().get_mut("int").unwrap().push(arg.clone());
                        }
                    } 
            }
            curr_func = expr[1].clone();
        } else if is_ret(expr.clone()) {
            stmts.push(parse_ret(expr.clone(), &vars, &curr_func));
        } else if is_fnc_call(expr.clone()) {
            stmts.push(parse_fnc_call(&expr[0], &expr[2..expr.len()-1]));
        } else {
            exit(11);
        }
        expr.clear();
        i += (j-i)+1;  
    }
    return stmts;
}
