//! Lexer / tokenizer with quoting and variable expansion.
//!
//! Tokenization deliberately does *not* perform expansion. It segments the
//! input into raw words (quotes and `$` preserved) and operators. Expansion
//! (`$VAR`, `${VAR}`, `$?`, `$$`, `~`, field splitting) is applied later, per
//! command, by [`expand_word`] so that variables assigned earlier in the same
//! line are visible to later commands — mirroring POSIX `sh` semantics.

/// A redirection operator (with optional file-descriptor prefix).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirOp {
    /// `<`
    In,
    /// `>`
    Out,
    /// `>>`
    App,
    /// `&>`
    Both,
    /// `>& target` (duplicate stdout onto target fd)
    DupOut,
    /// `<& target` (duplicate stdin from target fd)
    DupIn,
    /// `FD<`, `FD>`, `FD>>`, `FD>&`, `FD<&`
    Fd(u32, Box<RedirOp>),
}

/// A lexical token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    /// A bare or quoted word (raw; expanded at execution time).
    Word(String),
    /// `|`
    Pipe,
    /// `;`
    Semi,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `&`
    Amp,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// A redirection operator.
    Redir(RedirOp),
}

/// Variable lookup function used during expansion (lifetime-parameterised so
/// a closure borrowing `&Shell` can be passed in without requiring `'static`).
pub type Lookup<'a> = dyn Fn(&str) -> Option<String> + 'a;

fn expand_var(name: &str, lookup: &Lookup, last_status: i32) -> Option<String> {
    match name {
        "?" => return Some(last_status.to_string()),
        "$" => return Some(std::process::id().to_string()),
        _ => {}
    }

    if let Some((var_name, default)) = name.split_once(":-") {
        let val = lookup(var_name);
        if let Some(v) = val {
            if !v.is_empty() {
                return Some(v);
            }
        }
        return Some(default.to_string());
    }

    if let Some((var_name, alt)) = name.split_once(":+") {
        let val = lookup(var_name);
        if let Some(v) = val {
            if !v.is_empty() {
                return Some(alt.to_string());
            }
        }
        return Some(String::new());
    }

    lookup(name)
}

/// Expand a `$name`, `${name}`, `$?`, `$$` occurrence starting at `s[i]`.
/// Returns `(value, new_index_after_closing_char)`.
fn read_parameter(
    s: &[char],
    i: usize,
    lookup: &Lookup,
    last_status: i32,
) -> Option<(String, usize)> {
    if i + 1 >= s.len() {
        return None;
    }
    let c = s[i + 1];
    if c == '?' {
        return expand_var("?", lookup, last_status).map(|v| (v, i + 2));
    }
    if c == '$' {
        return expand_var("$", lookup, last_status).map(|v| (v, i + 2));
    }
    if c == '{' {
        let mut j = i + 2;
        let mut name = String::new();
        while j < s.len() && s[j] != '}' {
            name.push(s[j]);
            j += 1;
        }
        if j < s.len() {
            return expand_var(&name, lookup, last_status).map(|v| (v, j + 1));
        }
        return None;
    }
    if c.is_ascii_alphabetic() || c == '_' {
        let mut j = i + 1;
        let mut name = String::new();
        while j < s.len() && (s[j].is_ascii_alphanumeric() || s[j] == '_') {
            name.push(s[j]);
            j += 1;
        }
        return expand_var(&name, lookup, last_status).map(|v| (v, j));
    }
    None
}

fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
}

/// Read a single-quoted segment (literal until the next `'`).
fn read_single_quoted(s: &[char], i: usize) -> Result<(String, usize), String> {
    let mut out = String::new();
    let mut j = i + 1;
    while j < s.len() {
        if s[j] == '\'' {
            return Ok((out, j + 1));
        }
        out.push(s[j]);
        j += 1;
    }
    Err("unterminated single quote".into())
}

/// Read a double-quoted segment, expanding `$` and a few escapes.
fn read_double_quoted(
    s: &[char],
    i: usize,
    lookup: &Lookup,
    last_status: i32,
) -> Result<(String, usize), String> {
    let mut out = String::new();
    let mut j = i + 1;
    while j < s.len() {
        let c = s[j];
        if c == '"' {
            return Ok((out, j + 1));
        }
        if c == '\\' && j + 1 < s.len() {
            let n = s[j + 1];
            match n {
                '"' | '\\' | '$' | '`' => out.push(n),
                _ => {
                    out.push('\\');
                    out.push(n);
                }
            }
            j += 2;
            continue;
        }
        if c == '$' {
            if let Some((val, ni)) = read_parameter(s, j, lookup, last_status) {
                out.push_str(&val);
                j = ni;
                continue;
            }
        }
        out.push(c);
        j += 1;
    }
    Err("unterminated double quote".into())
}

/// Read a raw word (quotes and `$` preserved) starting at `s[i]`.
///
/// Only segments the input on unquoted/unescaped whitespace and operators;
/// the returned text keeps quotes and escapes so [`expand_word`] can resolve
/// them later.
fn read_raw_word(s: &[char], i: usize) -> Result<(String, usize), String> {
    let mut out = String::new();
    let mut j = i;
    while j < s.len() {
        let c = s[j];
        if c.is_whitespace() || matches!(c, '|' | ';' | '&' | '(' | ')' | '<' | '>') {
            break;
        }
        if c == '\'' {
            out.push(c);
            j += 1;
            while j < s.len() && s[j] != '\'' {
                out.push(s[j]);
                j += 1;
            }
            if j < s.len() {
                out.push(s[j]);
                j += 1;
            } else {
                return Err("unterminated single quote".into());
            }
            continue;
        }
        if c == '"' {
            out.push(c);
            j += 1;
            while j < s.len() && s[j] != '"' {
                out.push(s[j]);
                j += 1;
            }
            if j < s.len() {
                out.push(s[j]);
                j += 1;
            } else {
                return Err("unterminated double quote".into());
            }
            continue;
        }
        if c == '\\' && j + 1 < s.len() {
            out.push(c);
            out.push(s[j + 1]);
            j += 2;
            continue;
        }
        out.push(c);
        j += 1;
    }
    Ok((out, j))
}

/// Expand a single raw word into one or more fields, resolving quotes, `$`
/// expansion and `~`, and applying POSIX field splitting on the result of
/// unquoted expansions. Called by the executor right before each command runs
/// so that variables set earlier in the same line are visible.
pub fn expand_word(
    raw: &str,
    lookup: &Lookup,
    last_status: i32,
) -> Result<Vec<String>, String> {
    let s: Vec<char> = raw.chars().collect();
    let n = s.len();
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut started = false;

    let flush = |cur: &mut String, words: &mut Vec<String>, started: &mut bool| {
        if *started {
            words.push(std::mem::take(cur));
            *started = false;
        }
    };

    let mut j = 0;
    while j < n {
        let c = s[j];
        if c == '\'' {
            let (q, ni) = read_single_quoted(&s, j)?;
            cur.push_str(&q);
            started = true;
            j = ni;
            continue;
        }
        if c == '"' {
            let (q, ni) = read_double_quoted(&s, j, lookup, last_status)?;
            cur.push_str(&q);
            started = true;
            j = ni;
            continue;
        }
        if c == '\\' && j + 1 < n {
            cur.push(s[j + 1]);
            started = true;
            j += 2;
            continue;
        }
        if c == '$' {
            if let Some((val, ni)) = read_parameter(&s, j, lookup, last_status) {
                if val.is_empty() {
                    j = ni;
                    continue;
                }
                for part in val.split_whitespace() {
                    if started {
                        cur.push_str(part);
                    } else {
                        cur = part.to_string();
                        started = true;
                    }
                    flush(&mut cur, &mut words, &mut started);
                }
                j = ni;
                continue;
            }
            cur.push('$');
            started = true;
            j += 1;
            continue;
        }
        if c == '~' && !started {
            cur.push_str(&home_dir());
            started = true;
            j += 1;
            continue;
        }
        if c.is_whitespace() {
            flush(&mut cur, &mut words, &mut started);
            j += 1;
            continue;
        }
        cur.push(c);
        started = true;
        j += 1;
    }

    flush(&mut cur, &mut words, &mut started);
    if words.is_empty() {
        words.push(String::new());
    }
    Ok(words)
}


/// Expand a raw word into exactly one string (used for redirection targets
/// and assignment values). Returns an error on ambiguous (multi-field)
/// results.
pub fn expand_one(raw: &str, lookup: &Lookup, last_status: i32) -> Result<String, String> {
    let words = expand_word(raw, lookup, last_status)?;
    if words.len() == 1 {
        Ok(words.into_iter().next().unwrap())
    } else {
        Err("ambiguous redirect".into())
    }
}

/// Tokenize the input line into a vector of raw tokens (no expansion).
pub fn tokenize(input: &str) -> Result<Vec<Tok>, String> {
    let s: Vec<char> = input.chars().collect();
    let n = s.len();
    let mut toks = Vec::new();
    let mut i = 0;
    let mut after_whitespace = true;

    while i < n {
        let c = s[i];
        if c.is_whitespace() {
            i += 1;
            after_whitespace = true;
            continue;
        }

        if c == '#' && after_whitespace {
            break;
        }
        after_whitespace = false;


        if c == '|' {
            if i + 1 < n && s[i + 1] == '|' {
                toks.push(Tok::OrOr);
                i += 2;
            } else {
                toks.push(Tok::Pipe);
                i += 1;
            }
            continue;
        }
        if c == '&' {
            if i + 1 < n && s[i + 1] == '&' {
                toks.push(Tok::AndAnd);
                i += 2;
            } else if i + 1 < n && s[i + 1] == '>' {
                toks.push(Tok::Redir(RedirOp::Both));
                i += 2;
            } else {
                toks.push(Tok::Amp);
                i += 1;
            }
            continue;
        }
        if c == ';' {
            toks.push(Tok::Semi);
            i += 1;
            continue;
        }
        if c == '(' {
            toks.push(Tok::LParen);
            i += 1;
            continue;
        }
        if c == ')' {
            toks.push(Tok::RParen);
            i += 1;
            continue;
        }
        if c == '<' {
            if i + 1 < n && s[i + 1] == '&' {
                toks.push(Tok::Redir(RedirOp::DupIn));
                i += 2;
            } else {
                toks.push(Tok::Redir(RedirOp::In));
                i += 1;
            }
            continue;
        }
        if c == '>' {
            if i + 1 < n && s[i + 1] == '&' {
                toks.push(Tok::Redir(RedirOp::DupOut));
                i += 2;
            } else if i + 1 < n && s[i + 1] == '>' {
                toks.push(Tok::Redir(RedirOp::App));
                i += 2;
            } else {
                toks.push(Tok::Redir(RedirOp::Out));
                i += 1;
            }
            continue;
        }

        // File-descriptor prefixed redirections: `2>`, `2>>`, `2>&`, `2<&`.
        if c.is_ascii_digit() {
            let mut k = i;
            let mut fd = 0u32;
            while k < n && s[k].is_ascii_digit() {
                fd = fd * 10 + s[k].to_digit(10).unwrap();
                k += 1;
            }
            if k < n && (s[k] == '>' || s[k] == '<') {
                let op = if s[k] == '<' {
                    if k + 1 < n && s[k + 1] == '&' {
                        RedirOp::Fd(fd, Box::new(RedirOp::DupIn))
                    } else {
                        RedirOp::Fd(fd, Box::new(RedirOp::In))
                    }
                } else if k + 1 < n && s[k + 1] == '&' {
                    RedirOp::Fd(fd, Box::new(RedirOp::DupOut))
                } else if k + 1 < n && s[k + 1] == '>' {
                    RedirOp::Fd(fd, Box::new(RedirOp::App))
                } else {
                    RedirOp::Fd(fd, Box::new(RedirOp::Out))
                };
                let skip = match &op {
                    RedirOp::Fd(_, inner) => match **inner {
                        RedirOp::DupIn | RedirOp::DupOut => 2,
                        RedirOp::App => 2,
                        _ => 1,
                    },
                    _ => 1,
                };
                toks.push(Tok::Redir(op));
                i = k + skip;
                continue;
            }
        }

        let (w, ni) = read_raw_word(&s, i)?;
        toks.push(Tok::Word(w));
        i = ni;
    }

    Ok(toks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("echo hello world").unwrap();
        assert_eq!(
            tokens,
            vec![
                Tok::Word("echo".to_string()),
                Tok::Word("hello".to_string()),
                Tok::Word("world".to_string())
            ]
        );
    }

    #[test]
    fn test_tokenize_pipeline_and_redirection() {
        let tokens = tokenize("ls -l | grep test > output.txt").unwrap();
        assert_eq!(
            tokens,
            vec![
                Tok::Word("ls".to_string()),
                Tok::Word("-l".to_string()),
                Tok::Pipe,
                Tok::Word("grep".to_string()),
                Tok::Word("test".to_string()),
                Tok::Redir(RedirOp::Out),
                Tok::Word("output.txt".to_string())
            ]
        );
    }

    #[test]
    fn test_expand_word() {
        let lookup = |name: &str| match name {
            "FOO" => Some("bar".to_string()),
            _ => None,
        };
        let expanded = expand_word("\"$FOO/baz\"", &lookup, 0).unwrap();
        assert_eq!(expanded, vec!["bar/baz".to_string()]);
    }

    #[test]
    fn test_expand_parameter_defaults() {
        let lookup = |name: &str| match name {
            "FOO" => Some("bar".to_string()),
            _ => None,
        };
        let fallback = expand_word("\"${UNSET:-fallback}\"", &lookup, 0).unwrap();
        assert_eq!(fallback, vec!["fallback".to_string()]);

        let set_val = expand_word("\"${FOO:-fallback}\"", &lookup, 0).unwrap();
        assert_eq!(set_val, vec!["bar".to_string()]);

        let alt_val = expand_word("\"${FOO:+alternative}\"", &lookup, 0).unwrap();
        assert_eq!(alt_val, vec!["alternative".to_string()]);
    }
}


