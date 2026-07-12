use super::*;
use super::lexer::{Redirect, Token};

/// Builds a `Command` from the buffered words/redirects and pushes it onto `commands`.
fn finalize(words: &mut Vec<Word>, redirects: &mut Vec<Redirect>, commands: &mut Vec<Command>) {
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

fn finalize_pipeline(
    words: &mut Vec<Word>,
    redirects: &mut Vec<Redirect>,
    commands: &mut Vec<Command>,
) -> Option<Pipeline> {
    finalize(words, redirects, commands);
    if commands.is_empty() {
        None
    } else {
        Some(Pipeline {
            commands: std::mem::take(commands),
        })
    }
}

/// Parses a full line (possibly containing `;`, `&&`, `||`, `|`, and a
/// trailing `&`) into a `CommandList`.
pub fn parse(tokens: Vec<Token>) -> CommandList {
    let mut items: Vec<(AndOrList, Option<ListOp>)> = Vec::new();

    let mut commands: Vec<Command> = Vec::new();
    let mut words: Vec<Word> = Vec::new();
    let mut redirects: Vec<Redirect> = Vec::new();
    let mut background = false;

    for token in tokens {
        match token {
            Token::Word(w) => words.push(w),
            Token::Redirect(r) => redirects.push(r),
            Token::Pipe => finalize(&mut words, &mut redirects, &mut commands),
            Token::Semi => {
                close_item(&mut words, &mut redirects, &mut commands, &mut background, Some(ListOp::Seq), &mut items);
            }
            Token::And => {
                close_item(&mut words, &mut redirects, &mut commands, &mut background, Some(ListOp::And), &mut items);
            }
            Token::Or => {
                close_item(&mut words, &mut redirects, &mut commands, &mut background, Some(ListOp::Or), &mut items);
            }
            Token::Background => {
                background = true;
                close_item(&mut words, &mut redirects, &mut commands, &mut background, None, &mut items);
            }
        }
    }
    // Trailing pipeline with no following operator.
    close_item(&mut words, &mut redirects, &mut commands, &mut background, None, &mut items);

    CommandList { items }
}

fn close_item(
    words: &mut Vec<Word>,
    redirects: &mut Vec<Redirect>,
    commands: &mut Vec<Command>,
    background: &mut bool,
    op: Option<ListOp>,
    items: &mut Vec<(AndOrList, Option<ListOp>)>,
) {
    if let Some(pipeline) = finalize_pipeline(words, redirects, commands) {
        items.push((
            AndOrList {
                pipeline,
                background: *background,
            },
            op,
        ));
    }
    *background = false;
}
