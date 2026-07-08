use super::*;
use super::lexer::{Redirect, Token};

/// Builds a `Command` from the buffered words/redirects and pushes it onto `commands`.
fn finalize(words: &mut Vec<String>, redirects: &mut Vec<Redirect>, commands: &mut Vec<Command>) {
    if words.is_empty() {
        return;
    }
    let program = words.remove(0);

    // `&>` / `&>>` target both streams: expand into two explicit redirects.
    // (`-1` is the sentinel produced by the lexer for "both"; fd 0 is stdin.)
    let mut expanded: Vec<Redirect> = Vec::new();
    for r in redirects.drain(..) {
        if r.fd == -1 {
            expanded.push(Redirect {
                fd: 1,
                append: r.append,
                target: r.target.clone(),
            });
            expanded.push(Redirect {
                fd: 2,
                append: r.append,
                target: r.target.clone(),
            });
        } else {
            expanded.push(r);
        }
    }

    commands.push(Command {
        program,
        args: std::mem::take(words),
        redirects: expanded,
        heredoc: None,
    });
}

pub fn parse(tokens: Vec<Token>) -> Pipeline {
    let mut commands: Vec<Command> = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut redirects: Vec<Redirect> = Vec::new();

    for token in tokens {
        match token {
            Token::Word(w) => words.push(w),
            Token::Pipe => finalize(&mut words, &mut redirects, &mut commands),
            Token::Redirect(r) => redirects.push(r),
        }
    }
    finalize(&mut words, &mut redirects, &mut commands);

    Pipeline { commands }
}
