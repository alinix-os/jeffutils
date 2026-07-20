mod builtin;
mod completion;
mod executor;
mod parser;
mod shell;
mod utils;

use std::io::{BufRead, IsTerminal};
use std::sync::atomic::{AtomicBool, Ordering};

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::hint::HistoryHinter;
use rustyline::history::DefaultHistory;
use rustyline::{Config, CompletionType, Editor};

use crate::builtin::run_jeofetch;
use crate::completion::JshHelper;
use crate::parser::lexer::RedirectTarget;
use crate::shell::ShellState;

static SIGINT_FLAG: AtomicBool = AtomicBool::new(false);

extern "C" fn sigint_handler(_sig: i32) {
    SIGINT_FLAG.store(true, Ordering::SeqCst);
}

/// Expands `!!`, `!n`, and `!prefix` history references in a raw input
/// line, using the rustyline history as the source of past commands.
/// Runs before tokenizing, exactly like bash's history expansion.
fn expand_history_refs(line: &str, history: &rustyline::history::DefaultHistory) -> String {
    use rustyline::history::History;

    if !line.contains('!') {
        return line.to_string();
    }

    let mut out = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '!' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('!') => {
                chars.next();
                if let Ok(Some(entry)) = history.get(
                    history.len().wrapping_sub(1),
                    rustyline::history::SearchDirection::Forward,
                ) {
                    out.push_str(&entry.entry);
                } else {
                    out.push_str("!!");
                }
            }
            Some('$') => {
                chars.next();
                if let Ok(Some(entry)) = history.get(
                    history.len().wrapping_sub(1),
                    rustyline::history::SearchDirection::Forward,
                ) {
                    if let Some(last_arg) = entry.entry.split_whitespace().last() {
                        out.push_str(last_arg);
                    } else {
                        out.push_str("!$");
                    }
                } else {
                    out.push_str("!$");
                }
            }
            Some(d) if d.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let idx = num.parse::<usize>().unwrap_or(0);
                if idx >= 1
                    && let Ok(Some(entry)) =
                        history.get(idx - 1, rustyline::history::SearchDirection::Forward)
                {
                    out.push_str(&entry.entry);
                } else {
                    out.push('!');
                    out.push_str(&num);
                }
            }
            Some(c) if c.is_alphabetic() => {
                let mut prefix = String::new();
                while let Some(&pc) = chars.peek() {
                    if pc.is_alphanumeric() || pc == '_' {
                        prefix.push(pc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let found = (0..history.len())
                    .rev()
                    .find_map(|i| {
                        history
                            .get(i, rustyline::history::SearchDirection::Forward)
                            .ok()
                            .flatten()
                            .filter(|e| e.entry.starts_with(&prefix))
                            .map(|e| e.entry.to_string())
                    });
                match found {
                    Some(entry) => out.push_str(&entry),
                    None => {
                        out.push('!');
                        out.push_str(&prefix);
                    }
                }
            }
            _ => out.push('!'),
        }
    }
    out
}

/// Reads heredoc bodies for every heredoc redirect found in `list`, prompting
/// interactively via `read_more` (used for the REPL) or consuming lines from
/// `lines` (used for non-interactive script execution). Returns one body per
/// `AndOrList` item (parallel to `list.items`), `None` where there's no heredoc.
fn collect_heredocs(
    list: &parser::CommandList,
    mut read_more: impl FnMut(&str) -> Option<String>,
) -> Vec<Option<String>> {
    let mut bodies = Vec::with_capacity(list.items.len());
    for (andor, _op) in &list.items {
        let mut delim: Option<String> = None;
        for cmd in &andor.pipeline.commands {
            for r in &cmd.redirects {
                if let RedirectTarget::Heredoc(d) = &r.target {
                    delim = Some(d.clone());
                }
            }
        }
        if let Some(delim) = delim {
            let mut body = String::new();
            loop {
                match read_more("> ") {
                    Some(l) if l.trim() == delim => break,
                    Some(l) => {
                        body.push_str(&crate::utils::expand_env_vars(&l));
                        body.push('\n');
                    }
                    None => break,
                }
            }
            bodies.push(Some(body));
        } else {
            bodies.push(None);
        }
    }
    bodies
}

/// Parses and executes one raw input line against `state`, using
/// `read_more` to pull additional lines for heredoc bodies when needed.
pub fn run_line_with(state: &mut ShellState, line: &str, mut read_more: impl FnMut(&str) -> Option<String>) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }

    let tokens = crate::parser::lexer::tokenize(line);
    let list = crate::parser::parser::parse(tokens);
    if list.items.is_empty() {
        return;
    }

    let heredoc_bodies = collect_heredocs(&list, &mut read_more);

    crate::executor::run_command_list(state, &list, &heredoc_bodies);
}

fn run_interactive(mut state: ShellState) {
    unsafe {
        libc::signal(libc::SIGINT, sigint_handler as *const () as usize);
        
        // Ignore job control signals so the shell doesn't get suspended
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        libc::signal(libc::SIGTTIN, libc::SIG_IGN);
        libc::signal(libc::SIGTSTP, libc::SIG_IGN);
        libc::signal(libc::SIGQUIT, libc::SIG_IGN);

        // Put ourselves in our own process group if we are the foreground process
        let pid = libc::getpid();
        let _ = libc::setpgid(pid, pid);
        let _ = libc::tcsetpgrp(libc::STDIN_FILENO, pid);
    }

    // Configure history file path ~/.jsh-history
    let history_path = state.home_dir.join(".jsh-history");

    // Use List completion (Bash style) so it doesn't auto-insert choices you have to delete
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .completion_prompt_limit(100)
        .bracketed_paste(true)
        .build();

    let mut rl = Editor::<JshHelper, DefaultHistory>::with_config(config)
        .expect("Erro ao inicializar editor de linha");

    struct UpArrowHandler;
    impl rustyline::ConditionalEventHandler for UpArrowHandler {
        fn handle(&self, _evt: &rustyline::Event, _n: rustyline::RepeatCount, _positive: bool, ctx: &rustyline::EventContext) -> Option<rustyline::Cmd> {
            if ctx.line().is_empty() {
                Some(rustyline::Cmd::PreviousHistory)
            } else {
                Some(rustyline::Cmd::HistorySearchBackward)
            }
        }
    }
    struct DownArrowHandler;
    impl rustyline::ConditionalEventHandler for DownArrowHandler {
        fn handle(&self, _evt: &rustyline::Event, _n: rustyline::RepeatCount, _positive: bool, ctx: &rustyline::EventContext) -> Option<rustyline::Cmd> {
            if ctx.line().is_empty() {
                Some(rustyline::Cmd::NextHistory)
            } else {
                Some(rustyline::Cmd::HistorySearchForward)
            }
        }
    }
    struct RightArrowHandler;
    impl rustyline::ConditionalEventHandler for RightArrowHandler {
        fn handle(&self, _evt: &rustyline::Event, _n: rustyline::RepeatCount, _positive: bool, ctx: &rustyline::EventContext) -> Option<rustyline::Cmd> {
            if ctx.pos() == ctx.line().len() {
                Some(rustyline::Cmd::CompleteHint)
            } else {
                None
            }
        }
    }

    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Up, rustyline::Modifiers::empty()),
        rustyline::EventHandler::Conditional(Box::new(UpArrowHandler)),
    );
    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Down, rustyline::Modifiers::empty()),
        rustyline::EventHandler::Conditional(Box::new(DownArrowHandler)),
    );
    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Right, rustyline::Modifiers::empty()),
        rustyline::EventHandler::Conditional(Box::new(RightArrowHandler)),
    );
    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Char('f'), rustyline::Modifiers::CTRL),
        rustyline::Cmd::CompleteHint,
    );

    let helper = JshHelper {
        hinter: HistoryHinter::new(),
        completer: FilenameCompleter::new(),
        aliases: state.aliases.clone(),
        shell_vars: state.shell_vars.clone(),
        functions: state.functions.clone(),
    };
    rl.set_helper(Some(helper));

    // Load global history
    if history_path.exists() {
        let _ = rl.load_history(&history_path);
    }

    // Helper: minimal percent-encoding suitable for the path component of OSC 7.
    fn encode_osc7_path(p: &std::path::Path) -> String {
        let mut out = String::new();
        for b in p.display().to_string().bytes() {
            match b {
                b'%' => out.push_str("%25"),
                b' ' => out.push_str("%20"),
                b'#' => out.push_str("%23"),
                b'?' => out.push_str("%3F"),
                _ if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.' || b == b'/' => {
                    out.push(b as char);
                }
                _ => out.push_str(&format!("%{:02X}", b)),
            }
        }
        out
    }

    loop {
        // Emit OSC 7 to inform terminal emulator of the current working directory.
        // Written to stderr so it bypasses rustyline's alternate-screen buffer
        // and is reliably picked up by GNOME Terminal / VTE for Ctrl+Shift+T.
        if let Ok(pwd) = std::env::current_dir() {
            let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
            use std::io::Write;
            let encoded_pwd = encode_osc7_path(&pwd);
            eprint!("\x1b]7;file://{}{}\x1b\\", hostname, encoded_pwd);
            let _ = std::io::stderr().flush();
        }

        let prompt = state.render_prompt();
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Re-read .jshrc if it was edited since last load (and
                // HOT_RELOAD=true). Done here, after the line was entered,
                // so edits made while the prompt was waiting take effect on
                // the very next command instead of one command later.
                state.maybe_hot_reload();

                let expanded_line = expand_history_refs(line, rl.history());

                rl.add_history_entry(line).ok();
                let _ = rl.save_history(&history_path);

                let show_timing = state.get_var("SHOW_TIMING") != "false";
                let start_time = std::time::Instant::now();
                run_line_with(&mut state, &expanded_line, |prompt| rl.readline(prompt).ok());
                if SIGINT_FLAG.swap(false, Ordering::SeqCst) {
                    println!("^C");
                }
                if show_timing {
                    let elapsed = start_time.elapsed();
                    if elapsed.as_secs_f64() >= 2.0 {
                        eprintln!("\x1B[38;5;240m(⏳ demorou {:.1}s)\x1B[0m", elapsed.as_secs_f64());
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

/// Non-interactive mode: reads a full script (from a file argument or piped
/// stdin) and runs it through `ShellState::run_script_text`, supporting
/// `;`/`&&`/`||`/pipes/heredocs/function definitions without requiring a TTY.
fn run_script<R: BufRead>(mut state: ShellState, mut reader: R) {
    let mut content = String::new();
    let _ = reader.read_to_string(&mut content);
    state.run_script_text(&content);
    std::process::exit(state.last_exit_status);
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("jsh", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let mut state = ShellState::new();

    let args: Vec<String> = std::env::args().collect();

    let mut cmd_string: Option<String> = None;
    let mut script_path: Option<String> = None;
    let mut script_args: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-c" {
            if i + 1 < args.len() {
                cmd_string = Some(args[i + 1].clone());
                if i + 2 < args.len() {
                    state.arg0 = args[i + 2].clone();
                    script_args = args[i + 3..].to_vec();
                }
                break;
            } else {
                eprintln!("jsh: -c: option requires an argument");
                std::process::exit(2);
            }
        } else if arg == "-l" || arg == "--login" {
            // Ignore/skip login shell flags but don't treat them as script files
            i += 1;
        } else if arg.starts_with('-') {
            // Skip other options to avoid failing
            i += 1;
        } else {
            // First non-option argument is the script file path
            script_path = Some(arg.clone());
            script_args = args[i + 1..].to_vec();
            break;
        }
    }

    state.set_positional_args(script_args);

    if let Some(command_string) = cmd_string {
        state.load_jshrc();
        state.run_script_text(&command_string);
        std::process::exit(state.last_exit_status);
    }

    if let Some(path) = script_path {
        state.arg0 = path.clone();
        state.load_jshrc();
        match std::fs::File::open(&path) {
            Ok(f) => run_script(state, std::io::BufReader::new(f)),
            Err(e) => {
                eprintln!("jsh: {}: {}", path, e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Load config from .jshrc
    state.load_jshrc();

    if !std::io::stdin().is_terminal() {
        run_script(state, std::io::stdin().lock());
        return;
    }

    // Run jeofetch on init, but only in an interactive terminal session.
    // Skip it for non-tty invocations like `jsh -c "..."`, piped stdin
    // (e.g. `!pwd` inside Claude), or when stdout is redirected — there
    // jeofetch would just be noise in captured output.
    if state.init_info && std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        run_jeofetch();
    }

    run_interactive(state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_execution() {
        let mut state = ShellState::new();
        run_line_with(&mut state, "TEST_VAR=hello", |_| None);
        assert_eq!(state.get_var("TEST_VAR"), "hello");
    }
}
