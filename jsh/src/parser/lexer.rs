#[derive(Debug, Clone)]
pub enum RedirectTarget {
    File(String),
    Fd(i32),
    /// Heredoc: the string is the delimiter; the body is read by the REPL.
    Heredoc(String),
    /// Here-string: the string is the literal content fed to stdin (`<<<`).
    HereString(String),
}

#[derive(Debug, Clone)]
pub struct Redirect {
    /// File descriptor: 0 = stdin, 1 = stdout, 2 = stderr, -1 = both (from `&>`).
    pub fd: i32,
    pub append: bool,
    pub target: RedirectTarget,
}

#[derive(Debug, Clone)]
pub enum Token {
    Word(String),
    Pipe,
    Redirect(Redirect),
}

/// Splits a command line into tokens, recognizing pipes and shell-style
/// redirections (`>`, `>>`, `2>`, `2>>`, `1>`, `1>>`, `&>`, `&>>`), input
/// redirection (`<`), here-strings (`<<<`) and heredocs (`<<`).
pub fn tokenize(input: &str) -> Vec<Token> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < parts.len() {
        let part = parts[i];

        if part == "|" {
            tokens.push(Token::Pipe);
        } else if let Some((fd, append, rest)) = match_redirect_prefix(part) {
            // The target is either glued to the operator (e.g. `2>/dev/null`)
            // or is the following whitespace-separated word.
            let target_str = if rest.is_empty() {
                i += 1;
                if i >= parts.len() {
                    // Dangling redirection with no target: ignore it.
                    i += 1;
                    continue;
                }
                parts[i].to_string()
            } else {
                rest.to_string()
            };

            let (target, real_fd) = match fd {
                -2 => (RedirectTarget::HereString(target_str), 0),
                -3 => (
                    RedirectTarget::Heredoc(
                        target_str.trim_matches(|c| c == '\'' || c == '"').to_string(),
                    ),
                    0,
                ),
                _ => {
                    if let Some(stripped) = target_str.strip_prefix('&') {
                        match stripped.parse::<i32>() {
                            Ok(n) => (RedirectTarget::Fd(n), fd),
                            Err(_) => (RedirectTarget::File(target_str), fd),
                        }
                    } else {
                        (RedirectTarget::File(target_str), fd)
                    }
                }
            };

            tokens.push(Token::Redirect(Redirect {
                fd: real_fd,
                append,
                target,
            }));
        } else {
            tokens.push(Token::Word(part.to_string()));
        }

        i += 1;
    }

    tokens
}

/// Returns `(fd, append, rest)` when `part` begins with a redirection operator.
/// `rest` is whatever follows the operator within the same word (may be empty
/// when the target is a separate word). `fd` sentinels: `-1` = both streams
/// (`&>`), `-2` = here-string, `-3` = heredoc.
fn match_redirect_prefix(part: &str) -> Option<(i32, bool, &str)> {
    // Here-string: `<<< word`
    if part.starts_with("<<<") {
        return Some((-2, false, &part[3..]));
    }
    // Heredoc: `<< DELIM` (tab stripping `<<-` is ignored)
    if part.starts_with("<<") {
        return Some((-3, false, &part[2..]));
    }
    if let Some(rest) = part.strip_prefix("<") {
        return Some((0, false, rest));
    }
    if let Some(rest) = part.strip_prefix("&>>") {
        Some((-1, true, rest))
    } else if let Some(rest) = part.strip_prefix("&>") {
        Some((-1, false, rest))
    } else if let Some(rest) = part.strip_prefix("2>>") {
        Some((2, true, rest))
    } else if let Some(rest) = part.strip_prefix("1>>") {
        Some((1, true, rest))
    } else if let Some(rest) = part.strip_prefix(">>") {
        Some((1, true, rest))
    } else if let Some(rest) = part.strip_prefix("2>") {
        Some((2, false, rest))
    } else if let Some(rest) = part.strip_prefix("1>") {
        Some((1, false, rest))
    } else if let Some(rest) = part.strip_prefix(">") {
        Some((1, false, rest))
    } else {
        None
    }
}
