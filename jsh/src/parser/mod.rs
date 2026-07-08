pub mod lexer;
pub mod parser;

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub redirects: Vec<lexer::Redirect>,
    /// Body of a heredoc (`<< DELIM`); `None` unless the command reads a heredoc.
    pub heredoc: Option<String>,
}

#[derive(Debug)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}
