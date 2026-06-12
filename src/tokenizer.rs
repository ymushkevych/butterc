use std::{collections::HashMap, fs, process::exit};

fn tokenize_lists(chars: &Vec<char>, mut buf: Vec<char>, mut i: usize) -> Vec<String> {
    while i < chars.len()
    && chars[i] != ']' {
        if chars[i] != ' ' {
            if chars[i] == '"' {
                buf.push(chars[i]);
                let v = tokenize_strings(&chars, buf.clone(), i);
                for _char in v[0].chars() {
                    buf.push(_char);
                }
                i = v[1].parse::<usize>().unwrap();
            } else {
                buf.push(chars[i]);
                i += 1;
            }
        }
    }
    if i < chars.len() {
        buf.push(chars[i]);
    } else {
        println!("\x1b[1mSyntaxError\x1b[0m: Expected `]` to end list");
        exit(1);
    }
    i += 1;
    
    return vec![buf.iter().collect(), i.to_string()];
}

fn tokenize_strings(chars: &Vec<char>, mut buf: Vec<char>, mut i: usize) -> Vec<String> {
    while i < chars.len()
    && chars[i] != '"' {
        if chars[i] == '%' {
            buf.push('\\'); //convert % to \ for escape characters in NASM
        } else if chars[i] == '\\' {
            buf.push('\\');
            buf.push('\\');
        } else {
            buf.push(chars[i]);
        }
        i += 1;   
    }
    if i < chars.len() {
        buf.push(chars[i]);
    } else {
        println!("\x1b[1mSyntaxError\x1b[0m: Expected `\"` to end string");
        exit(1);
    }
    i += 1;

    return vec![buf.iter().collect(), i.to_string()];
}

pub fn tokenize(code: String, _flags: &[String]) -> Vec<String> {
    let mut in_string: bool = false;
    let mut in_list: bool = false;
    let mut in_comment: bool = false;
    let mut token_pool= HashMap::new();
    token_pool.insert(";".to_string(), "SEMI".to_string());
    token_pool.insert("+".to_string(), "PLUS".to_string());
    token_pool.insert("=".to_string(), "EQ".to_string());
    token_pool.insert("-".to_string(), "MIN".to_string());
    token_pool.insert(">".to_string(), "GT".to_string());
    token_pool.insert("*".to_string(), "TIMES".to_string());
    token_pool.insert("/".to_string(), "DIV".to_string());
    token_pool.insert("{".to_string(), "CURLO".to_string());
    token_pool.insert("}".to_string(), "CURLC".to_string());
    token_pool.insert("(".to_string(), "PARO".to_string());
    token_pool.insert(")".to_string(), "PARC".to_string());
    token_pool.insert(",".to_string(), "COMMA".to_string());
    token_pool.insert(":".to_string(), "COL".to_string());
    token_pool.insert("&".to_string(), "AMP".to_string());
    //read file into a char vector
    let mut chars: Vec<char> = vec![];
    let lines:Result<String, std::io::Error> = fs::read_to_string(&code);
    if let Ok(line) = lines {
        for _char in line.chars() {
            chars.push(_char);
        }
    }
    if chars == ['\n']
    || chars.len() == 0
    || chars.iter().all(|x| [' ', '\n'].contains(x)) {
        println!("\x1b[1mEmptyFileError\x1b[0m: Empty file provided");
        println!("All `.btr` files must contain a `main` function");
        println!("Try inserting `int fnc main() {}`", "{}");
        exit(1);
    }

    //read chars into a buffer and tokenize

    let mut buf: Vec<char> = vec![];
    let mut tokens: Vec<String> = vec![];

    let mut i: usize = 0;
    while i < chars.len() {
        let _char: char = chars[i];
        if !in_comment {
            if _char == '~' {
                in_comment = true;
                i += 1;
                continue;
            }
            if !in_string {
                if _char == '"' {
                    in_string = true;
                    buf.push(chars[i]);
                    i += 1;
                    continue;
                }
                if !in_list {
                    if _char == '[' {
                        in_list = true;
                        i += 1;
                        continue;
                    }
                    if _char.is_alphabetic() {
                        buf.push(_char);
                        i += 1;
                        while i < chars.len() && (chars[i].is_alphanumeric() 
                        || chars[i] == '_') {
                            buf.push(chars[i]);
                            i += 1;
                        }
                        tokens.push(buf.iter().collect());
                        buf.clear();
                    }
                    else if _char.is_numeric() {
                        buf.push(_char);
                        i += 1;
                        while i < chars.len() && chars[i].is_numeric() {
                            buf.push(chars[i]);
                            i += 1;
                        }
                        tokens.push(buf.iter().collect());
                        buf.clear();
                    }
                    else if token_pool.contains_key(&_char.to_string()) {
                        let token: String = token_pool.get(&_char.to_string()).cloned().unwrap_or_default();
                        if token == "MIN" {
                            if i + 1 < chars.len() &&
                            token_pool.get(&chars[i+1].to_string()).cloned().unwrap_or_default() == "GT" { 
                                tokens.push("LET".to_string()); //since '->' is a single token for var dec, convert sequence of "MIN" and "GT" into "LET"
                                i += 1;
                            } else {
                                tokens.push(token); 
                            }
                        } else {
                            tokens.push(token); 
                        }
                        i += 1;
                } else {
                    if _char == '\n' {
                        if tokens[tokens.len()-1] == "SEMI" 
                        || tokens[tokens.len()-1] == "CURLO"
                        || tokens[tokens.len()-1] == "CURLC" {
                            i+=1;
                        } else {
                            println!("\x1b[1mSyntaxError\x1b[0m: Expected `;`");
                            exit(1);
                        }
                    } else {
                        i+=1;
                    }
                }
                } 
                else {
                    let v = tokenize_lists(&chars, buf.clone(), i);
                    tokens.push(v[0].clone());
                    buf.clear();
                    i = v[1].parse::<usize>().unwrap();
                    in_list = false;
                    continue;
                }
            }
            else {
                let v = tokenize_strings(&chars, buf.clone(), i);
                tokens.push(v[0].clone());
                buf.clear();
                i = v[1].parse::<usize>().unwrap();
                in_string = false;
                continue;
            }
    } else {
        if _char == '\n'  {
            in_comment = false;
        }
        i += 1;
    }
}
    return tokens;
}
