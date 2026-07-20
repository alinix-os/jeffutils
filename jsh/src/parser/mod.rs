pub mod lexer;
pub mod parser;

/// A single piece of a word. A word is built from one or more segments so
/// that quoting/expansion rules can differ within the same token
/// (e.g. `foo-$(bar)-"baz $X"`).
#[derive(Debug, Clone)]
pub enum WordSegment {
    /// Literal text (from single quotes, or unquoted/double-quoted text
    /// after backslash-escapes have been resolved).
    Literal(String),
    /// `$VAR` / `${VAR}` reference, to be resolved against shell/env vars.
    VarExpand(String),
    /// `$(...)` or `` `...` ``: command substitution. Holds the raw source
    /// text to be re-tokenized/parsed/executed at expansion time.
    CommandSubst(String),
    /// Leading `~` or `~/...`, to be resolved against $HOME.
    Tilde(String),
    /// `${NAME:+word}` / `${NAME:-word}` parameter expansion:
    /// `(name, '+' or '-', word)`.
    ParamOp(String, char, String),
}

#[derive(Debug, Clone, Default)]
pub struct Word {
    pub segments: Vec<WordSegment>,
    /// True if the whole word came from single/double quotes (disables
    /// globbing and word-splitting on the expanded result).
    pub quoted: bool,
}

impl Word {
    pub fn literal(s: impl Into<String>) -> Self {
        Word {
            segments: vec![WordSegment::Literal(s.into())],
            quoted: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    pub program: Word,
    pub args: Vec<Word>,
    /// Leading `NAME=value` assignments scoped to this command.
    pub env_vars: Vec<(String, String)>,
    pub redirects: Vec<lexer::Redirect>,
    /// Body of a heredoc (`<< DELIM`); `None` unless the command reads a heredoc.
    pub heredoc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListOp {
    And,
    Or,
    Seq,
}

#[derive(Debug, Clone)]
pub struct AndOrList {
    pub pipeline: Pipeline,
    pub background: bool,
}

/// A full parsed line: a sequence of pipelines joined by `;`, `&&`, `||`.
/// `items[i].1` is the operator that joins `items[i]` to `items[i+1]`
/// (`None` for the last item).
#[derive(Debug, Clone, Default)]
pub struct CommandList {
    pub items: Vec<(AndOrList, Option<ListOp>)>,
}

/// Fully-expanded command (post variable/glob/tilde/command-subst expansion),
/// ready for the executor. Distinct from the parser's `Command` (which still
/// holds unexpanded `Word`s).
#[derive(Debug, Clone)]
pub struct ExpandedCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub redirects: Vec<lexer::Redirect>,
    pub heredoc: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExpandedPipeline {
    pub commands: Vec<ExpandedCommand>,
}
