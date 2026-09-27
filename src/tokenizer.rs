use std::{fs, process::exit};


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
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `]` to end list");
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
        eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `\"` to end string");
        exit(1);
    }
    i += 1;

    return vec![buf.iter().collect(), i.to_string()];
}

pub fn tokenize(code: String, _flags: &[String]) -> Vec<String> {
    let mut in_string: bool = false;
    let mut in_list: bool = false;
    let mut in_comment: bool = false;

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
        eprintln!("\x1b[1mEmptyFileError\x1b[0m: Empty file provided");
        eprintln!("All `.btr` files must contain a `main` function");
        eprintln!("Try inserting `int fnc main() {}`", "{}");
        exit(1);
    }

    //read chars into a buffer and tokenize

    let mut buf: Vec<char> = vec![];
    let mut tokens: Vec<String> = vec![];

    let mut i: usize = 0;
    while i < chars.len() {
        let _char: char = chars[i];
        if !in_comment {
            if _char == '%' {
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
                    } else if _char == '\n' {
                        if tokens.len() == 0 || 
                        tokens[tokens.len()-1] == ';'.to_string() || 
                        tokens[tokens.len()-1] == '{'.to_string() ||
                        tokens[tokens.len()-1] == '}'.to_string(){
                            i+=1;
                        } else {
                            eprintln!("\x1b[1mSyntaxError\x1b[0m: Expected `;`");
                            exit(1);
                        }
                    } else {
                        if _char == '-' && chars[i+1] == '>' {
                            tokens.push("->".to_string());
                            i+=1;
                        } else if _char == '>' && chars[i+1] == '=' {
                            tokens.push(">=".to_string());
                            i+=1;
                        } else if _char == '<' && chars[i+1] == '=' {
                            tokens.push("<=".to_string());
                            i+=1;
                        } else if !_char.is_ascii_whitespace() {
                            tokens.push(_char.to_string());
                        }
                        i += 1;
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
