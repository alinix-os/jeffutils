//! Recursive-descent parser turning tokens into the AST.

use crate::ast::{Connector, Job, Pipeline, Program, Redir, SimpleCommand};
use crate::lexer::{RedirOp, Tok};

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

fn is_assignment(word: &str) -> Option<(String, String)> {
    let eq = word.find('=')?;
    if eq == 0 {
        return None;
    }
    let (name, value) = word.split_at(eq);
    let value = &value[1..];
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return None,
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    Some((name.to_string(), value.to_string()))
}

fn parse_fd(target: &str) -> Result<u32, String> {
    target
        .parse::<u32>()
        .map_err(|_| format!("bad file descriptor: {target}"))
}

fn build_redir(op: &RedirOp, target: &str) -> Result<Redir, String> {
    Ok(match op {
        RedirOp::In => Redir::In(target.to_string()),
        RedirOp::Out => Redir::Out(target.to_string(), false),
        RedirOp::App => Redir::Out(target.to_string(), true),
        RedirOp::Both => Redir::Both(target.to_string()),
        RedirOp::DupOut => Redir::Dup(1, parse_fd(target)?),
        RedirOp::DupIn => Redir::Dup(0, parse_fd(target)?),
        RedirOp::Fd(fd, inner) => match &**inner {
            RedirOp::In => Redir::In(target.to_string()),
            RedirOp::Out => {
                if *fd == 2 {
                    Redir::Err(target.to_string(), false)
                } else {
                    Redir::Out(target.to_string(), false)
                }
            }
            RedirOp::App => {
                if *fd == 2 {
                    Redir::Err(target.to_string(), true)
                } else {
                    Redir::Out(target.to_string(), true)
                }
            }
            RedirOp::Both => Redir::Both(target.to_string()),
            RedirOp::DupOut | RedirOp::DupIn => Redir::Dup(*fd, parse_fd(target)?),
            RedirOp::Fd(..) => return Err("nested fd redirection".into()),
        },
    })
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse_program(&mut self) -> Result<Program, String> {
        let mut jobs = Vec::new();
        loop {
            match self.peek() {
                None | Some(Tok::RParen) => break,
                _ => {}
            }
            let job = self.parse_job()?;
            jobs.push(job);
        }
        Ok(Program { jobs })
    }

    fn parse_job(&mut self) -> Result<Job, String> {
        let pipeline = self.parse_pipeline()?;

        let mut background = false;
        if matches!(self.peek(), Some(Tok::Amp)) {
            self.next();
            background = true;
        }

        let connector = match self.peek() {
            Some(Tok::Semi) => {
                self.next();
                Connector::Seq
            }
            Some(Tok::AndAnd) => {
                self.next();
                Connector::And
            }
            Some(Tok::OrOr) => {
                self.next();
                Connector::Or
            }
            _ => Connector::Last,
        };

        Ok(Job {
            pipeline,
            background,
            connector,
        })
    }

    fn parse_pipeline(&mut self) -> Result<Pipeline, String> {
        let mut commands = Vec::new();
        commands.push(self.parse_simple_command()?);
        while matches!(self.peek(), Some(Tok::Pipe)) {
            self.next();
            commands.push(self.parse_simple_command()?);
        }
        Ok(Pipeline { commands })
    }

    fn parse_simple_command(&mut self) -> Result<SimpleCommand, String> {
        let mut cmd = SimpleCommand::default();
        let mut seen_program = false;

        loop {
            match self.peek() {
                Some(Tok::Word(w)) => {
                    if !seen_program {
                        if let Some((name, value)) = is_assignment(&w) {
                            cmd.assigns.push((name, value));
                            self.next();
                            continue;
                        }
                    }
                    seen_program = true;
                    cmd.argv.push(w.clone());
                    self.next();
                }
                Some(Tok::Redir(_)) => {
                    let op = match self.next() {
                        Some(Tok::Redir(op)) => op,
                        _ => unreachable!(),
                    };
                    let target = match self.next() {
                        Some(Tok::Word(t)) => t,
                        _ => {
                            return Err("syntax error: expected redirection target".into())
                        }
                    };
                    cmd.redirs.push(build_redir(&op, &target)?);
                }
                _ => break,
            }
        }

        Ok(cmd)
    }
}

/// Parse a token stream into a [`Program`].
pub fn parse(toks: Vec<Tok>) -> Result<Program, String> {
    let mut p = Parser { toks, pos: 0 };
    p.parse_program()
}
