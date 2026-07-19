use std::env;
use std::process;

fn usage() {
    eprintln!("Usage: calculate EXPRESSION");
    eprintln!("  Supports: +, -, *, /, %, ==, !=, <, >, <=, >=, |, length STR");
    process::exit(1);
}

fn version() {
    println!("calculate 0.1.0");
    process::exit(0);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        version();
    }

    let expr: String = args.join(" ");
    match eval_expr(&expr) {
        Ok(val) => println!("{}", val),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn eval_expr(expr: &str) -> Result<String, String> {
    let tokens = tokenize(expr)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    let (result, _) = parse_concat(&tokens, 0)?;
    Ok(result)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(i64),
    Str(String),
    Len,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Pipe,
    LParen,
    RParen,
}

fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        if chars[i] == '\'' || chars[i] == '"' {
            let quote = chars[i];
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        '\\' => s.push('\\'),
                        c if c == quote => s.push(c),
                        c => {
                            s.push('\\');
                            s.push(c);
                        }
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i >= chars.len() {
                return Err("Unterminated string".to_string());
            }
            i += 1;
            tokens.push(Token::Str(s));
            continue;
        }

        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            let num: i64 = num_str
                .parse()
                .map_err(|_| format!("Invalid number: {}", num_str))?;
            tokens.push(Token::Num(num));
            continue;
        }

        if chars[i].is_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i].is_alphabetic() {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            match word.as_str() {
                "length" | "len" => tokens.push(Token::Len),
                other => tokens.push(Token::Str(other.to_string())),
            }
            continue;
        }

        match chars[i] {
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '%' => tokens.push(Token::Percent),
            '=' => {
                i += 1;
                if i < chars.len() && chars[i] == '=' {
                    tokens.push(Token::Eq);
                } else {
                    tokens.push(Token::Eq);
                    continue;
                }
            }
            '!' => {
                i += 1;
                if i < chars.len() && chars[i] == '=' {
                    tokens.push(Token::Ne);
                } else {
                    return Err("Unexpected character: !".to_string());
                }
            }
            '<' => {
                i += 1;
                if i < chars.len() && chars[i] == '=' {
                    tokens.push(Token::Le);
                } else {
                    tokens.push(Token::Lt);
                    continue;
                }
            }
            '>' => {
                i += 1;
                if i < chars.len() && chars[i] == '=' {
                    tokens.push(Token::Ge);
                } else {
                    tokens.push(Token::Gt);
                    continue;
                }
            }
            '|' => tokens.push(Token::Pipe),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            c => return Err(format!("Unexpected character: {}", c)),
        }
        i += 1;
    }
    Ok(tokens)
}

fn parse_concat(tokens: &[Token], pos: usize) -> Result<(String, usize), String> {
    let (mut result, mut pos) = parse_comparison(tokens, pos)?;

    while pos < tokens.len() && tokens[pos] == Token::Pipe {
        pos += 1;
        let (right, new_pos) = parse_comparison(tokens, pos)?;
        result = format!("{}{}", result, right);
        pos = new_pos;
    }

    Ok((result, pos))
}

fn parse_comparison(tokens: &[Token], pos: usize) -> Result<(String, usize), String> {
    let (left_str, mut pos) = parse_add_sub(tokens, pos)?;

    if pos < tokens.len() {
        match &tokens[pos] {
            Token::Eq | Token::Ne | Token::Lt | Token::Gt | Token::Le | Token::Ge => {
                let op = tokens[pos].clone();
                pos += 1;
                let (right_str, new_pos) = parse_add_sub(tokens, pos)?;
                pos = new_pos;

                let result = match op {
                    Token::Eq => left_str == right_str,
                    Token::Ne => left_str != right_str,
                    Token::Lt => left_str < right_str,
                    Token::Gt => left_str > right_str,
                    Token::Le => left_str <= right_str,
                    Token::Ge => left_str >= right_str,
                    _ => unreachable!(),
                };
                return Ok((result.to_string(), pos));
            }
            _ => {}
        }
    }

    Ok((left_str, pos))
}

fn parse_add_sub(tokens: &[Token], pos: usize) -> Result<(String, usize), String> {
    let (mut left, mut pos) = parse_mul_div(tokens, pos)?;

    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Plus | Token::Minus => {
                let op = tokens[pos].clone();
                pos += 1;
                let (right, new_pos) = parse_mul_div(tokens, pos)?;
                pos = new_pos;

                let left_val: i64 = left.parse().map_err(|_| format!("Not a number: {}", left))?;
                let right_val: i64 = right.parse().map_err(|_| format!("Not a number: {}", right))?;

                left = match op {
                    Token::Plus => (left_val + right_val).to_string(),
                    Token::Minus => (left_val - right_val).to_string(),
                    _ => unreachable!(),
                };
            }
            _ => break,
        }
    }

    Ok((left, pos))
}

fn parse_mul_div(tokens: &[Token], pos: usize) -> Result<(String, usize), String> {
    let (mut left, mut pos) = parse_unary(tokens, pos)?;

    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Star | Token::Slash | Token::Percent => {
                let op = tokens[pos].clone();
                pos += 1;
                let (right, new_pos) = parse_unary(tokens, pos)?;
                pos = new_pos;

                let left_val: i64 = left.parse().map_err(|_| format!("Not a number: {}", left))?;
                let right_val: i64 = right.parse().map_err(|_| format!("Not a number: {}", right))?;

                left = match op {
                    Token::Star => (left_val * right_val).to_string(),
                    Token::Slash => {
                        if right_val == 0 {
                            return Err("Division by zero".to_string());
                        }
                        (left_val / right_val).to_string()
                    }
                    Token::Percent => {
                        if right_val == 0 {
                            return Err("Division by zero".to_string());
                        }
                        (left_val % right_val).to_string()
                    }
                    _ => unreachable!(),
                };
            }
            _ => break,
        }
    }

    Ok((left, pos))
}

fn parse_unary(tokens: &[Token], pos: usize) -> Result<(String, usize), String> {
    if pos < tokens.len() {
        match &tokens[pos] {
            Token::Minus => {
                let pos = pos + 1;
                let (val, pos) = parse_primary(tokens, pos)?;
                let n: i64 = val.parse().map_err(|_| format!("Not a number: {}", val))?;
                return Ok(((-n).to_string(), pos));
            }
            Token::Len => {
                let pos = pos + 1;
                let (val, pos) = parse_primary(tokens, pos)?;
                return Ok((val.len().to_string(), pos));
            }
            _ => {}
        }
    }
    parse_primary(tokens, pos)
}

fn parse_primary(tokens: &[Token], pos: usize) -> Result<(String, usize), String> {
    if pos >= tokens.len() {
        return Err("Unexpected end of expression".to_string());
    }

    match &tokens[pos] {
        Token::Num(n) => Ok((n.to_string(), pos + 1)),
        Token::Str(s) => Ok((s.clone(), pos + 1)),
        Token::LParen => {
            let pos = pos + 1;
            let (result, pos) = parse_concat(tokens, pos)?;
            if pos >= tokens.len() || tokens[pos] != Token::RParen {
                return Err("Missing closing parenthesis".to_string());
            }
            Ok((result, pos + 1))
        }
        t => Err(format!("Unexpected token: {:?}", t)),
    }
}
