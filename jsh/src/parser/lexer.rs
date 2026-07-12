use super::{Word, WordSegment};

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
    Word(Word),
    Pipe,
    Redirect(Redirect),
    /// `;`
    Semi,
    /// `&&`
    And,
    /// `||`
    Or,
    /// trailing `&` (background)
    Background,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
            tokens: Vec::new(),
        }
    }

    fn run(mut self) -> Vec<Token> {
        loop {
            self.skip_spaces();
            let Some(&c) = self.chars.peek() else { break };

            match c {
                '#' => {
                    // Comment: skip to end of line.
                    while self.chars.next().is_some() {}
                    break;
                }
                '|' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'|') {
                        self.chars.next();
                        self.tokens.push(Token::Or);
                    } else {
                        self.tokens.push(Token::Pipe);
                    }
                }
                '&' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'&') {
                        self.chars.next();
                        self.tokens.push(Token::And);
                    } else if self.chars.peek() == Some(&'>') {
                        self.chars.next();
                        let append = self.chars.peek() == Some(&'>');
                        if append {
                            self.chars.next();
                        }
                        self.skip_spaces();
                        let target = self.read_word();
                        self.tokens.push(Token::Redirect(Redirect {
                            fd: -1,
                            append,
                            target: RedirectTarget::File(flatten_literal(&target)),
                        }));
                    } else {
                        self.tokens.push(Token::Background);
                    }
                }
                ';' => {
                    self.chars.next();
                    self.tokens.push(Token::Semi);
                }
                '<' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'<') {
                        self.chars.next();
                        if self.chars.peek() == Some(&'<') {
                            self.chars.next();
                            self.skip_spaces();
                            let target = self.read_word();
                            self.tokens.push(Token::Redirect(Redirect {
                                fd: 0,
                                append: false,
                                target: RedirectTarget::HereString(flatten_literal(&target)),
                            }));
                        } else {
                            // `<<-` tab-stripping form: treat like `<<`.
                            if self.chars.peek() == Some(&'-') {
                                self.chars.next();
                            }
                            self.skip_spaces();
                            let target = self.read_word();
                            let delim = flatten_literal(&target)
                                .trim_matches(|c| c == '\'' || c == '"')
                                .to_string();
                            self.tokens.push(Token::Redirect(Redirect {
                                fd: 0,
                                append: false,
                                target: RedirectTarget::Heredoc(delim),
                            }));
                        }
                    } else {
                        self.skip_spaces();
                        let target = self.read_word();
                        self.tokens.push(Token::Redirect(Redirect {
                            fd: 0,
                            append: false,
                            target: RedirectTarget::File(flatten_literal(&target)),
                        }));
                    }
                }
                '>' => {
                    self.chars.next();
                    let append = self.chars.peek() == Some(&'>');
                    if append {
                        self.chars.next();
                    }
                    self.skip_spaces();
                    let target = self.read_word();
                    self.tokens.push(Token::Redirect(Redirect {
                        fd: 1,
                        append,
                        target: redirect_target_from_word(&target),
                    }));
                }
                '0'..='9' => {
                    // Might be `N>` / `N>>` / `N<`; otherwise it's a normal word.
                    if let Some((fd, append, is_input)) = self.peek_numeric_redirect() {
                        self.skip_spaces();
                        let target = self.read_word();
                        self.tokens.push(Token::Redirect(Redirect {
                            fd,
                            append,
                            target: if is_input {
                                RedirectTarget::File(flatten_literal(&target))
                            } else {
                                redirect_target_from_word(&target)
                            },
                        }));
                    } else {
                        let word = self.read_word();
                        self.tokens.push(Token::Word(word));
                    }
                }
                _ => {
                    let word = self.read_word();
                    if !word.segments.is_empty() {
                        self.tokens.push(Token::Word(word));
                    }
                }
            }
        }

        self.tokens
    }

    fn skip_spaces(&mut self) {
        while matches!(self.chars.peek(), Some(c) if c.is_whitespace()) {
            self.chars.next();
        }
    }

    /// Looks ahead for a digit-prefixed redirect operator (`2>`, `2>>`, `0<`)
    /// without consuming input unless it matches. Returns `(fd, append, is_input)`.
    fn peek_numeric_redirect(&mut self) -> Option<(i32, bool, bool)> {
        let mut lookahead = self.chars.clone();
        let mut digits = String::new();
        while let Some(&c) = lookahead.peek() {
            if c.is_ascii_digit() {
                digits.push(c);
                lookahead.next();
            } else {
                break;
            }
        }
        if digits.is_empty() {
            return None;
        }
        match lookahead.peek() {
            Some('>') => {
                let fd: i32 = digits.parse().ok()?;
                for _ in 0..digits.len() {
                    self.chars.next();
                }
                self.chars.next(); // '>'
                let append = self.chars.peek() == Some(&'>');
                if append {
                    self.chars.next();
                }
                Some((fd, append, false))
            }
            Some('<') => {
                let fd: i32 = digits.parse().ok()?;
                for _ in 0..digits.len() {
                    self.chars.next();
                }
                self.chars.next(); // '<'
                Some((fd, false, true))
            }
            _ => None,
        }
    }

    /// Reads one whitespace-delimited word, honoring quotes, backslash
    /// escapes, and `$VAR` / `${VAR}` / `$(...)` / backtick expansions.
    fn read_word(&mut self) -> Word {
        let mut segments: Vec<WordSegment> = Vec::new();
        let mut current = String::new();
        let mut any_quotes = false;
        let mut at_word_start = true;

        macro_rules! flush_literal {
            () => {
                if !current.is_empty() {
                    segments.push(WordSegment::Literal(std::mem::take(&mut current)));
                }
            };
        }

        loop {
            let Some(&c) = self.chars.peek() else { break };

            if c.is_whitespace() {
                break;
            }
            match c {
                '|' | '&' | ';' | '<' | '>' => break,
                '\'' => {
                    any_quotes = true;
                    self.chars.next();
                    while let Some(&sc) = self.chars.peek() {
                        if sc == '\'' {
                            self.chars.next();
                            break;
                        }
                        current.push(sc);
                        self.chars.next();
                    }
                }
                '"' => {
                    any_quotes = true;
                    self.chars.next();
                    self.read_double_quoted(&mut current, &mut segments);
                }
                '\\' => {
                    self.chars.next();
                    if let Some(nc) = self.chars.next() {
                        current.push(nc);
                    }
                }
                '~' if at_word_start => {
                    self.chars.next();
                    flush_literal!();
                    let mut rest = String::from("~");
                    while let Some(&tc) = self.chars.peek() {
                        if tc.is_whitespace() || matches!(tc, '|' | '&' | ';' | '<' | '>' | '/') {
                            if tc == '/' {
                                rest.push(tc);
                                self.chars.next();
                            }
                            break;
                        }
                        rest.push(tc);
                        self.chars.next();
                    }
                    segments.push(WordSegment::Tilde(rest));
                }
                '$' => {
                    self.chars.next();
                    self.read_dollar(&mut current, &mut segments);
                }
                '`' => {
                    self.chars.next();
                    flush_literal!();
                    let mut body = String::new();
                    while let Some(&bc) = self.chars.peek() {
                        if bc == '`' {
                            self.chars.next();
                            break;
                        }
                        body.push(bc);
                        self.chars.next();
                    }
                    segments.push(WordSegment::CommandSubst(body));
                }
                _ => {
                    current.push(c);
                    self.chars.next();
                }
            }
            at_word_start = false;
        }

        flush_literal!();

        Word {
            segments,
            quoted: any_quotes,
        }
    }

    /// Reads the body of a double-quoted string, expanding `$VAR`/`$(...)`
    /// but treating everything else literally.
    fn read_double_quoted(&mut self, current: &mut String, segments: &mut Vec<WordSegment>) {
        loop {
            let Some(&c) = self.chars.peek() else { break };
            match c {
                '"' => {
                    self.chars.next();
                    break;
                }
                '\\' => {
                    self.chars.next();
                    if let Some(nc) = self.chars.next() {
                        // Only these are "special" escapes inside double quotes.
                        if matches!(nc, '"' | '\\' | '$' | '`') {
                            current.push(nc);
                        } else {
                            current.push('\\');
                            current.push(nc);
                        }
                    }
                }
                '$' => {
                    self.chars.next();
                    self.read_dollar(current, segments);
                }
                _ => {
                    current.push(c);
                    self.chars.next();
                }
            }
        }
    }

    /// Handles everything after a `$` has been consumed: `$VAR`, `${VAR}`,
    /// `$(...)`.
    fn read_dollar(&mut self, current: &mut String, segments: &mut Vec<WordSegment>) {
        match self.chars.peek() {
            Some('(') => {
                self.chars.next();
                if !current.is_empty() {
                    segments.push(WordSegment::Literal(std::mem::take(current)));
                }
                let mut depth = 1;
                let mut body = String::new();
                while let Some(&c) = self.chars.peek() {
                    self.chars.next();
                    if c == '(' {
                        depth += 1;
                        body.push(c);
                    } else if c == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        body.push(c);
                    } else {
                        body.push(c);
                    }
                }
                segments.push(WordSegment::CommandSubst(body));
            }
            Some('{') => {
                self.chars.next();
                if !current.is_empty() {
                    segments.push(WordSegment::Literal(std::mem::take(current)));
                }
                let mut body = String::new();
                let mut depth = 1;
                while let Some(&c) = self.chars.peek() {
                    self.chars.next();
                    if c == '{' {
                        depth += 1;
                        body.push(c);
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        body.push(c);
                    } else {
                        body.push(c);
                    }
                }
                segments.push(parse_brace_body(&body));
            }
            Some(&c) if c.is_alphanumeric() || c == '_' || c == '?' || c == '$' || c == '@' || c == '#' => {
                if !current.is_empty() {
                    segments.push(WordSegment::Literal(std::mem::take(current)));
                }
                if matches!(c, '?' | '$' | '@' | '#') {
                    self.chars.next();
                    segments.push(WordSegment::VarExpand(c.to_string()));
                } else {
                    let mut name = String::new();
                    while let Some(&c) = self.chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            name.push(c);
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    segments.push(WordSegment::VarExpand(name));
                }
            }
            _ => {
                // Lone `$` with nothing recognizable after it.
                current.push('$');
            }
        }
    }
}

/// Parses the body of a `${...}` expansion: either a plain `NAME` (possibly
/// `@`/`#`/digits), or `NAME:+word` / `NAME:-word` parameter expansion.
fn parse_brace_body(body: &str) -> WordSegment {
    if let Some(pos) = body.find(":+").filter(|&p| is_valid_param_name(&body[..p])) {
        return WordSegment::ParamOp(body[..pos].to_string(), '+', body[pos + 2..].to_string());
    }
    if let Some(pos) = body.find(":-").filter(|&p| is_valid_param_name(&body[..p])) {
        return WordSegment::ParamOp(body[..pos].to_string(), '-', body[pos + 2..].to_string());
    }
    WordSegment::VarExpand(body.to_string())
}

fn is_valid_param_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Collapses a `Word` produced for a redirect target/heredoc delimiter into
/// a plain string (no shell-var/glob expansion is performed here; that
/// happens later against `ShellState` for `$VAR` segments via a simple
/// literal join since redirect targets are commonly simple).
fn flatten_literal(word: &Word) -> String {
    let mut out = String::new();
    for seg in &word.segments {
        match seg {
            WordSegment::Literal(s) => out.push_str(s),
            WordSegment::Tilde(s) => out.push_str(s),
            WordSegment::VarExpand(name) => {
                out.push('$');
                out.push_str(name);
            }
            WordSegment::CommandSubst(s) => {
                out.push_str("$(");
                out.push_str(s);
                out.push(')');
            }
            WordSegment::ParamOp(name, op, word) => {
                out.push_str("${");
                out.push_str(name);
                out.push(':');
                out.push(*op);
                out.push_str(word);
                out.push('}');
            }
        }
    }
    out
}

fn redirect_target_from_word(word: &Word) -> RedirectTarget {
    let flat = flatten_literal(word);
    if let Some(stripped) = flat.strip_prefix('&') {
        if let Ok(n) = stripped.parse::<i32>() {
            return RedirectTarget::Fd(n);
        }
    }
    RedirectTarget::File(flat)
}

/// Splits a command line into tokens, recognizing pipes, list operators
/// (`;`, `&&`, `||`, trailing `&`), quoting, escapes, variable/command
/// substitution, and shell-style redirections.
pub fn tokenize(input: &str) -> Vec<Token> {
    Lexer::new(input).run()
}
