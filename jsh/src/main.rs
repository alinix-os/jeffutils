mod builtin;
mod completion;
mod executor;
mod parser;
mod shell;
mod utils;

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::hint::HistoryHinter;
use rustyline::history::DefaultHistory;
use rustyline::{Config, CompletionType, Editor};

use crate::builtin::{handle_builtin, is_builtin, run_jeofetch};
use crate::completion::JshHelper;
use crate::executor::execute_command;
use crate::parser::lexer::RedirectTarget;
use crate::shell::ShellState;
use crate::utils::expand_env_vars;

fn main() {
    let mut state = ShellState::new();

    // Load config from .jshrc
    state.load_jshrc();

    // Run jeofetch on init
    if state.init_info {
        run_jeofetch();
    }

    // Configure history file path ~/.jsh-history
    let history_path = state.home_dir.join(".jsh-history");

    // Use Circular completion (Vim style) for cycling through candidates
    let config = Config::builder()
        .completion_type(CompletionType::Circular)
        .completion_prompt_limit(100)
        .build();

    let mut rl = Editor::<JshHelper, DefaultHistory>::with_config(config)
        .expect("Erro ao inicializar editor de linha");
    let helper = JshHelper {
        hinter: HistoryHinter::new(),
        completer: FilenameCompleter::new(),
        aliases: state.aliases.clone(),
    };
    rl.set_helper(Some(helper));

    // Load global history
    if history_path.exists() {
        let _ = rl.load_history(&history_path);
    }

    loop {
        let prompt = state.render_prompt();
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(line).ok();
                let _ = rl.save_history(&history_path);

                let raw_args: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

                // Parse the line so we can support pipelines (`|`) and shell-style
                // redirections (`>`, `>>`, `2>`, `&>`, ...).
                let tokens = crate::parser::lexer::tokenize(&line);
                let pipeline = crate::parser::parser::parse(tokens);

                let has_redirects = pipeline.commands.iter().any(|c| !c.redirects.is_empty());
                let first_is_builtin =
                    pipeline.commands.first().map(|c| is_builtin(&c.program)).unwrap_or(false);

                // Read a heredoc body (`cmd << DELIM`) from subsequent lines if present.
                let mut heredoc_body: Option<String> = None;
                let mut heredoc_delim: Option<String> = None;
                for cmd in &pipeline.commands {
                    for r in &cmd.redirects {
                        if let RedirectTarget::Heredoc(d) = &r.target {
                            heredoc_delim = Some(d.clone());
                        }
                    }
                }
                if let Some(delim) = heredoc_delim {
                    let mut body = String::new();
                    loop {
                        match rl.readline("> ") {
                            Ok(l) => {
                                if l.trim() == delim {
                                    break;
                                }
                                body.push_str(&expand_env_vars(&l));
                                body.push('\n');
                            }
                            Err(_) => break,
                        }
                    }
                    heredoc_body = Some(body);
                }

                // Route through the executor when there is a pipe, or a redirection on an
                // external command. Builtins (cd, exit, ...) keep the in-process path.
                if pipeline.commands.len() > 1 || (has_redirects && !first_is_builtin) {
                    let mut expanded: Vec<crate::parser::Command> = Vec::new();
                    for cmd in &pipeline.commands {
                        let raw: Vec<String> =
                            std::iter::once(cmd.program.clone()).chain(cmd.args.clone()).collect();
                        let args = state.process_args(&raw);
                        if args.is_empty() {
                            continue;
                        }
                        let is_heredoc = cmd
                            .redirects
                            .iter()
                            .any(|r| matches!(r.target, RedirectTarget::Heredoc(_)));
                        expanded.push(crate::parser::Command {
                            program: args[0].clone(),
                            args: args[1..].to_vec(),
                            redirects: cmd.redirects.clone(),
                            heredoc: if is_heredoc { heredoc_body.clone() } else { None },
                        });
                    }
                    state.last_exit_status =
                        crate::executor::pipeline::execute(crate::parser::Pipeline { commands: expanded });
                } else {
                    // Single command, no redirection: keep builtin handling (cd, exit, etc.).
                    let args = state.process_args(&raw_args);

                    if let Some(status) = handle_builtin(&args, &mut state) {
                        state.last_exit_status = status;
                    } else {
                        state.last_exit_status = execute_command(&args);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!("Saindo do jsh...");
                break;
            }
            Err(err) => {
                println!("Erro: {:?}", err);
                break;
            }
        }
    }
}
