//! Abstract Syntax Tree for the shell.
//!
//! The grammar is intentionally small but POSIX-ish, mirroring what a
//! minimal `sh` (like the one shipped with coreutils / dash) supports:
//!
//! ```text
//! program   := list
//! list      := job (';' | '&' | '&&' | '||') list | job
//! job       := pipeline background?
//! pipeline  := command ('|' command)*
//! command   := assignment* word* redirection*
//! ```

/// A redirection attached to a simple command.
#[derive(Debug, Clone)]
pub enum Redir {
    /// `< file` — redirect stdin from a file.
    In(String),
    /// `> file` / `>> file` — redirect stdout to a file (append flag).
    Out(String, bool),
    /// `2> file` / `2>> file` — redirect stderr to a file (append flag).
    Err(String, bool),
    /// `&> file` — redirect both stdout and stderr to a file.
    Both(String),
    /// `2>&1` / `1>&2` / `>&2` — duplicate one fd onto another.
    Dup(u32, u32),
}

/// A simple command: program + arguments, plus redirections and
/// temporary environment assignments (`VAR=val cmd`).
#[derive(Debug, Clone, Default)]
pub struct SimpleCommand {
    /// Leading `NAME=VALUE` assignments (`export`-like, command scoped).
    pub assigns: Vec<(String, String)>,
    /// The program name and its arguments.
    pub argv: Vec<String>,
    /// Redirections applied before execution.
    pub redirs: Vec<Redir>,
}

/// A pipeline of one or more simple commands connected with `|`.
#[derive(Debug, Clone, Default)]
pub struct Pipeline {
    pub commands: Vec<SimpleCommand>,
}

/// How a job is joined to the next one in a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connector {
    /// `;` — always run the next job.
    Seq,
    /// `&&` — run the next job only on success.
    And,
    /// `||` — run the next job only on failure.
    Or,
    /// End of the list.
    Last,
}

/// A single job: a pipeline, whether it runs in the background, and how
/// it connects to the following job.
#[derive(Debug, Clone)]
pub struct Job {
    pub pipeline: Pipeline,
    pub background: bool,
    pub connector: Connector,
}

/// A parsed program: an ordered list of jobs.
#[derive(Debug, Clone, Default)]
pub struct Program {
    pub jobs: Vec<Job>,
}
