use std::{process::exit};

const TYPES: [&str; 4] = ["int", "vec", "str", "bool"];
const ATTRIBUTES: [&str; 2] = ["var", "const"];

fn parse_ret(expr: Vec<String>, int_vars: &Vec<String>) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["ret".to_string()]; 
    if expr.len() > 2 {
        if is_bin_expr(&expr[1..]) {
            stmt = parse_bin_expr(&expr[1..], int_vars, stmt);
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
        } else if int_vars.contains(&expr[1]) {
            stmt.push(format!("{}", expr[1].clone()));
        } else {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Only integer literals or variables evaluating to integer literals can be returned");
            exit(1);
        }
    }
    return stmt;
}

fn parse_inline_fnc_call(name: &String, args: &[String], mut asm: Vec<String>) -> Vec<String> {
    asm.push(format!("fcall {} {:?}", name, args[0..].concat()));

    return asm;
}

fn parse_fnc_call(name: &String, args: &[String]) -> Vec<String> {
    let stmt: Vec<String> = vec!["fcall".to_string(), name.to_string(), args[0..].concat()];

    return stmt;
}

fn parse_fnc_dec(name: &String, args: &[String], funcs: &Vec<String>) -> Vec<String> {
    let stmt: Vec<String> = vec!["fdec".to_string(), name.to_string()];
    if funcs.contains(name) {
        eprintln!("\x1b[1mParserError\x1b[0m: cannot redeclare existing functions");
        exit(1);
    }
    let binding = args.concat();
    let argv: Vec<&str> = binding.split("COMMA").collect();
    let mut args: Vec<String>  = vec![];
    if argv.len() > 0 && argv[0] != ""{
        if !is_int_lit(&argv[0].to_string()) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: First item in a list of arguments must always be the count");
            exit(1);
        }
        if argv.len() == 1 {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected arguments after count declaration");
            eprintln!("For a function with 0 arguments, an argument count is not needed");
            exit(1);
        }
        for i in (1..argv.len()).step_by(2) {
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
                args.push(arg[1].to_string());
            }
        }
        if (argv.len()-1) != argv[0].parse::<usize>().unwrap_or_default() {
            eprintln!("\x1b[1mParserError\x1b[0m: Amount of arguments specified does not equal the amount of arguments found");
            exit(1);
        }
    } else {
        args.push("NONE".to_string());
    }
    return stmt;
}

fn parse_var_ass(name: &String, value: &[String], consts: &Vec<String>, int_vars: &Vec<String>) -> Vec<String> {
    if is_const_var(name, consts) {
        eprintln!("\x1b[1mParserError\x1b[0m: Cannot reassign value to constant variable");
        exit(1);
    }
    let mut stmt: Vec<String> = vec!["vass".to_string(), name.to_string()];
    if value.len() > 1 {
        stmt = parse_bin_expr(value, int_vars, stmt);
    } else {
        if !is_int_lit(&value[0]) {
            exit(11);
        }
        stmt.push(value[0].to_string());
    }
    return stmt;
}

fn parse_var_dec(var_type: &String, attribute: &String, var: &[String], name: &String, consts: &Vec<String>, int_vars: &Vec<String> ) -> Vec<String> {
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
        stmt = parse_int(var, int_vars, stmt);
    } else if var_type == "vec" {
        exit(11);
    } else if var_type == "str" {
        exit(11);
    } else if var_type == "bool"{
        exit(11);
    }

    return stmt;
}

fn parse_bin_expr(expr: &[String], _int_vars: &Vec<String>, mut stmt: Vec<String>) -> Vec<String> {
    let mut expr = expr.to_vec();
    while expr.len() > 2 {
        /*if (!int_vars.contains(&expr[0]) && !is_int_lit(&expr[0]))
            || (!int_vars.contains(&expr[2]) && !is_int_lit(&expr[2])) {
            eprintln!("only integer literals, integer variables, and function are supported in binary expressions");
            exit(1);
        } */
        let i = find_bin_operator(&expr[0..]);
        if i == 0  {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Binary Expression cannot begin with a binary operator");
            exit(1);
        }
        let bin_expr_1_len =get_bin_expr_len(&expr, i, 0);

        if &expr[i] == &"PLUS".to_string() || &expr[i] == &"MIN".to_string() {
            if expr.len() == bin_expr_1_len {
                if &expr[i] == &"PLUS".to_string() {
                        stmt = parse_bin_add(&expr[0..bin_expr_1_len], stmt);
                        expr.drain(0..bin_expr_1_len);
                        expr.insert(0, "0".to_string());
                    } else {
                        stmt = parse_bin_sub(&expr[0..bin_expr_1_len], stmt);
                        expr.drain(0..bin_expr_1_len);
                        expr.insert(0, "0".to_string());
                    }
            } else {

                let j = bin_expr_1_len + 1;
                
                let bin_expr_2_len = get_bin_expr_len(&expr, j, i);
                /*if (!int_vars.contains(&expr[2]) && !is_int_lit(&expr[2]))
                    || (!int_vars.contains(&expr[4]) && !is_int_lit(&expr[4])) {
                    eprintln!("only integer literals and integer variables supported in binary expressions");
                    exit(1);
                } */
                if &expr[j] == &"PLUS".to_string() || &expr[j] == &"MIN".to_string() {
                    if &expr[i] == &"PLUS".to_string() {
                        stmt = parse_bin_add(&expr[0..bin_expr_1_len], stmt);
                        expr.drain(0..bin_expr_1_len);
                        expr.insert(0, "0".to_string());
                    } else {
                        stmt = parse_bin_sub(&expr[0..bin_expr_1_len], stmt);
                        expr.drain(0..bin_expr_1_len);
                        expr.insert(0, "0".to_string());
                    }
                } else if &expr[j] == &"TIMES".to_string() || &expr[j] == &"DIV".to_string() {
                    if &expr[j] == &"TIMES".to_string() {
                        //1 + 2 * 2
                        stmt = parse_bin_mult(&expr[i+1..bin_expr_2_len], stmt);
                        expr.drain(i+1..bin_expr_2_len);
                        expr.insert(i+1, "0".to_string());
                    } else {
                        stmt = parse_bin_div(&expr[i+1..bin_expr_2_len], stmt);
                        expr.drain(i+1..bin_expr_2_len);
                        expr.insert(i+1, "0".to_string());
                    }
                } else {
                    eprintln!("\x1b[1mSyntaxError\x1b[0m: Only `+`, `-`, `*`, `/` allowed in binary expressions");
                    exit(1);
                }  
            }
        } else if &expr[i] == &"TIMES".to_string() || &expr[i] == &"DIV".to_string() {
            if &expr[i] == &"TIMES".to_string() {
                stmt = parse_bin_mult(&expr[0..bin_expr_1_len], stmt);
                expr.drain(0..bin_expr_1_len);
                expr.insert(0, "0".to_string());
            } else if &expr[i] == &"DIV".to_string() {
                stmt = parse_bin_div(&expr[0..bin_expr_1_len], stmt);
                expr.drain(0..bin_expr_1_len);
                expr.insert(0, "0".to_string());
            } else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Only `+`, `-`, `*`, `/` allowed in binary expressions");
                exit(1);
            } 
        } else {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Only `+`, `-`, `*`, `/` allowed in binary expressions");
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

fn parse_print(prt_type: &String, expr: &[String], int_vars: &Vec<String>) -> Vec<String> {
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
                stmt = parse_bin_expr(&substring[0..], int_vars, stmt);
            } else if is_fnc_call(substring.clone()) {
                stmt = parse_inline_fnc_call(&substring[0], &substring[2..3], stmt);
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

fn parse_int(var: &[String], int_vars: &Vec<String>, mut stmt:Vec<String>) -> Vec<String> {
    if var.len() > 1 {
        if is_bin_expr(var){
            stmt = parse_bin_expr(var, int_vars, stmt);
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

fn parse_out(expr: Vec<String>, int_vars: &Vec<String>) -> Vec<String> {
    let mut stmt: Vec<String> = vec!["out".to_string()]; 
    if expr.len() > 2 {
        if is_bin_expr(&expr[1..]) {
            stmt = parse_bin_expr(&expr[1..], int_vars, stmt);
        } else {
            // I know there's only one argument, which must be a function
            if expr[2] == "PARO" {
                stmt = parse_inline_fnc_call(&expr[1], &expr[3..expr.len()-1], stmt);
            } else {
                eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected Parentheses for Function Arguments");
                exit(1);
            }
        }
    } else {
        if is_int_lit(&expr[1]) {
            stmt.push(expr[1].clone());
        } else if int_vars.contains(&expr[1]) {
            stmt.push(format!("{}", expr[1].clone()));
        } else {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: Only integer literals or variables/functions evaluating to integer literals can be used as exit codes");
            exit(1);
        }
    }
    return stmt;
}


fn get_bin_expr_len(bin_expr: &Vec<String>, start: usize, fancy_math_stuff: usize) -> usize {
    
    if find_bin_operator(&bin_expr[start+1..]) == 0 {
        return bin_expr[start+1..].len() + start + 1;
    } 

    return find_bin_operator(&bin_expr[start..]) + start + 1 - start + (start-fancy_math_stuff);

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
    if expr[1] != "PARO" {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `(` after function name in function call");
    }
    if expr[expr.len()-1] != "PARC" {
        eprintln!("\x1b[1mSyntaxError\x1b[0m:V Expected `)` after list of function arguments");
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
        return false;
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
    if expr[0] != "prt".to_string() 
    && expr[0] != "prf".to_string() 
    && expr[0] != "pre".to_string() {
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
            exit(1);
        }
    }
    if expr[1].len() == 2 {
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Print statement must have text to print");
        exit(1);
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


pub fn parse(tokens: Vec<String>, check_for_out: bool, mut funcs: Vec<String>) -> Vec<Vec<String>> {
    
    if check_for_out {
        if !tokens.contains(&"out".to_string()) {
            eprintln!("\x1b[1mSyntaxError\x1b[0m: No exit point specified\nExit points can be specified with `out \x1b[3mn\x1b[0m;`\nwhere \x1b[3mn\x1b[0m is an integer");
            exit(1);
        }
    }
    let mut int_vars: Vec<String> = vec![];
    let mut vars: Vec<String> = vec![];
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
        }
        else if is_out(expr.clone()) {
            stmts.push(parse_out(expr.clone(), &int_vars));
        } 
        else if is_var_dec(expr.clone()) {
            stmts.push(parse_var_dec(&expr[0], &expr[1], &expr[4..], &expr[2], &consts, &int_vars));
            if is_const_var(&expr[2], &consts) {
                consts.push(expr[2].clone());
            } else {
                vars.push(expr[2].clone());
            }
            if &expr[0] == "int" {int_vars.push(expr[2].clone());}
        }
        else if is_var_assignment(expr.clone()) {
            stmts.push(parse_var_ass(&expr[0], &expr[2..], &consts, &int_vars))
        } else if is_print(expr.clone()) {
            stmts.push(parse_print(&expr[0], &expr[1..]));
        } else if is_fnc_dec(expr.clone()) {
            if expr.len() == 4 {
                stmts.push(parse_fnc_dec(&expr[1], &[], &funcs));
            } else {
                stmts.push(parse_fnc_dec(&expr[1], &expr[3..expr.len()-1], &funcs));
            }
            funcs.push(expr[1].clone());
        } else if is_ret(expr.clone()) {
            stmts.push(parse_ret(expr.clone(), &int_vars));
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
