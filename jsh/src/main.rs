mod builtin;
mod completion;
mod executor;
mod parser;
mod shell;
mod utils;

use std::io::{BufRead, IsTerminal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Cmd, ConditionalEventHandler, Config, CompletionType, Editor, Event, EventContext, EventHandler, KeyCode, KeyEvent, Modifiers, RepeatCount, Movement};

use crate::builtin::run_jeofetch;
use crate::completion::JshHelper;
use crate::parser::lexer::RedirectTarget;
use crate::shell::ShellState;

static SIGINT_FLAG: AtomicBool = AtomicBool::new(false);



extern "C" fn sigint_handler(_sig: i32) {
    SIGINT_FLAG.store(true, Ordering::SeqCst);
}

/// Expands `!!`, `!n`, and `!prefix` history references in a raw input
/// line, using the history manager as the source of past commands.
/// Runs before tokenizing, exactly like bash's history expansion.
fn expand_history_refs(line: &str, history_mgr: &crate::shell::history::HistoryManager) -> String {
    if !line.contains('!') {
        return line.to_string();
    }

    let state = history_mgr.state.lock().unwrap();
    let entries = &state.entries;
    if entries.is_empty() {
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
                if let Some(entry) = entries.last() {
                    out.push_str(&entry.command);
                } else {
                    out.push_str("!!");
                }
            }
            Some('$') => {
                chars.next();
                if let Some(entry) = entries.last() {
                    if let Some(last_arg) = entry.command.split_whitespace().last() {
                        out.push_str(last_arg);
                    } else {
                        out.push_str("!$");
                    }
                } else {
                    out.push_str("!$");
                }
            }
            Some('?') => {
                chars.next();
                let mut pattern = String::new();
                while let Some(&pc) = chars.peek() {
                    if pc.is_alphanumeric() || pc == '_' || pc == '-' {
                        pattern.push(pc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let found = entries.iter().rev().find(|e| e.command.contains(&pattern));
                match found {
                    Some(entry) => out.push_str(&entry.command),
                    None => {
                        out.push_str("!?");
                        out.push_str(&pattern);
                    }
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
                if idx >= 1 && idx <= entries.len() {
                    out.push_str(&entries[idx - 1].command);
                } else {
                    out.push('!');
                    out.push_str(&num);
                }
            }
            Some(c) if c.is_alphabetic() => {
                let mut prefix = String::new();
                while let Some(&pc) = chars.peek() {
                    if pc.is_alphanumeric() || pc == '_' || pc == '-' {
                        prefix.push(pc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let found = entries.iter().rev().find(|e| e.command.starts_with(&prefix));
                match found {
                    Some(entry) => out.push_str(&entry.command),
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
                        body.push_str(&l);
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

/// Ensures `$PWD` in the environment matches the actual working directory at startup.
/// Process working directory (`current_dir()`) is authoritative because terminal emulators
/// and file managers perform `chdir()` before spawning the shell process.
/// If `$PWD` in the inherited environment is valid and canonicalizes to the same directory
/// as `current_dir()`, `$PWD` is left unchanged. Otherwise, `$PWD` is updated to match CWD.
fn sync_pwd() {
    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(_) => return,
    };

    if let Ok(pwd) = std::env::var("PWD") {
        let pwd_path = std::path::Path::new(&pwd);
        if pwd_path.is_dir() {
            if let (Ok(pwd_canon), Ok(cwd_canon)) = (pwd_path.canonicalize(), cwd.canonicalize()) {
                if pwd_canon == cwd_canon {
                    return;
                }
            }
        }
    }

    unsafe {
        std::env::set_var("PWD", &cwd);
    }
}

fn run_interactive(mut state: ShellState) {
    state.is_interactive = true;

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

    crate::utils::save_shell_termios();

    let config = Config::builder()
        .completion_type(CompletionType::Circular)
        .completion_prompt_limit(100)
        .bracketed_paste(true)
        .build();

    let mut rl = Editor::<JshHelper, DefaultHistory>::with_config(config)
        .expect("Erro ao inicializar editor de linha");

    struct CompleteHintHandler;
    impl ConditionalEventHandler for CompleteHintHandler {
        fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
            if ctx.pos() == ctx.line().len() {
                Some(Cmd::CompleteHint)
            } else {
                None
            }
        }
    }

    struct NavigationState {
        original_input: String,
        entries: Vec<String>,
        current_index: usize,
    }

    thread_local! {
        static NAVIGATION: std::cell::RefCell<Option<NavigationState>> = std::cell::RefCell::new(None);
    }

    struct UpArrowHandler {
        history_mgr: Arc<crate::shell::history::HistoryManager>,
        shell_vars: Arc<Mutex<HashMap<String, String>>>,
    }
    unsafe impl Send for UpArrowHandler {}
    unsafe impl Sync for UpArrowHandler {}
    impl ConditionalEventHandler for UpArrowHandler {
        fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
            let line = ctx.line();
            let cwd = {
                let vars = self.shell_vars.lock().unwrap();
                vars.get("PWD").cloned().unwrap_or_else(|| ".".to_string())
            };
            NAVIGATION.with(|cell| {
                let mut state_opt = cell.borrow_mut();
                let is_continuing = state_opt.as_ref()
                    .is_some_and(|s| s.current_index < s.entries.len() && s.entries[s.current_index] == line);

                if is_continuing {
                    let state = state_opt.as_mut().unwrap();
                    if state.current_index + 1 < state.entries.len() {
                        state.current_index += 1;
                        let next_cmd = state.entries[state.current_index].clone();
                        Some(Cmd::Replace(Movement::WholeBuffer, Some(next_cmd)))
                    } else {
                        None
                    }
                } else {
                    let entries = self.history_mgr.get_navigation_entries(line, &cwd);
                    if entries.is_empty() {
                        return None;
                    }
                    *state_opt = Some(NavigationState {
                        original_input: line.to_string(),
                        entries: entries.clone(),
                        current_index: 0,
                    });
                    Some(Cmd::Replace(Movement::WholeBuffer, Some(entries[0].clone())))
                }
            })
        }
    }

    struct DownArrowHandler;
    unsafe impl Send for DownArrowHandler {}
    unsafe impl Sync for DownArrowHandler {}
    impl ConditionalEventHandler for DownArrowHandler {
        fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
            let line = ctx.line();
            NAVIGATION.with(|cell| {
                let mut state_opt = cell.borrow_mut();
                let is_continuing = state_opt.as_ref()
                    .is_some_and(|s| s.current_index < s.entries.len() && s.entries[s.current_index] == line);

                if is_continuing {
                    let state = state_opt.as_mut().unwrap();
                    if state.current_index > 0 {
                        state.current_index -= 1;
                        let next_cmd = state.entries[state.current_index].clone();
                        Some(Cmd::Replace(Movement::WholeBuffer, Some(next_cmd)))
                    } else {
                        let original = state.original_input.clone();
                        *state_opt = None;
                        Some(Cmd::Replace(Movement::WholeBuffer, Some(original)))
                    }
                } else {
                    *state_opt = None;
                    None
                }
            })
        }
    }

    struct CtrlRHandler {
        history_mgr: Arc<crate::shell::history::HistoryManager>,
        shell_vars: Arc<Mutex<HashMap<String, String>>>,
    }
    unsafe impl Send for CtrlRHandler {}
    unsafe impl Sync for CtrlRHandler {}
    impl ConditionalEventHandler for CtrlRHandler {
        fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
            let cwd = {
                let vars = self.shell_vars.lock().unwrap();
                vars.get("PWD").cloned().unwrap_or_else(|| ".".to_string())
            };
            if let Ok(Some(selected)) = crate::shell::history::interactive_reverse_search(&self.history_mgr, &cwd) {
                Some(Cmd::Replace(Movement::WholeBuffer, Some(selected)))
            } else {
                Some(Cmd::Replace(Movement::WholeBuffer, Some(ctx.line().to_string())))
            }
        }
    }

    rl.bind_sequence(
        KeyEvent(KeyCode::Up, Modifiers::empty()),
        EventHandler::Conditional(Box::new(UpArrowHandler {
            history_mgr: state.history_mgr.clone(),
            shell_vars: state.shell_vars.clone(),
        })),
    );
    rl.bind_sequence(
        KeyEvent(KeyCode::Down, Modifiers::empty()),
        EventHandler::Conditional(Box::new(DownArrowHandler)),
    );
    rl.bind_sequence(
        KeyEvent(KeyCode::Char('r'), Modifiers::CTRL),
        EventHandler::Conditional(Box::new(CtrlRHandler {
            history_mgr: state.history_mgr.clone(),
            shell_vars: state.shell_vars.clone(),
        })),
    );
    rl.bind_sequence(
        KeyEvent(KeyCode::Right, Modifiers::empty()),
        EventHandler::Conditional(Box::new(CompleteHintHandler)),
    );
    rl.bind_sequence(
        KeyEvent(KeyCode::End, Modifiers::empty()),
        EventHandler::Conditional(Box::new(CompleteHintHandler)),
    );
    rl.bind_sequence(
        KeyEvent(KeyCode::Char('e'), Modifiers::CTRL),
        EventHandler::Conditional(Box::new(CompleteHintHandler)),
    );
    rl.bind_sequence(
        KeyEvent(KeyCode::Char('f'), Modifiers::CTRL),
        Cmd::CompleteHint,
    );

    let helper = JshHelper {
        history_mgr: state.history_mgr.clone(),
        completer: FilenameCompleter::new(),
        aliases: state.aliases.clone(),
        shell_vars: state.shell_vars.clone(),
        functions: state.functions.clone(),
    };
    rl.set_helper(Some(helper));

    loop {
        let prompt_clean = state.render_prompt_clean();
        let prompt_colored = state.render_prompt();
        crate::completion::CURRENT_COLORED_PROMPT.with(|cell| {
            *cell.borrow_mut() = prompt_colored;
        });

        NAVIGATION.with(|cell| {
            *cell.borrow_mut() = None;
        });

        let readline = rl.readline(&prompt_clean);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                state.maybe_hot_reload();

                let expanded_line = expand_history_refs(line, &state.history_mgr);

                let show_timing = state.get_var("SHOW_TIMING") != "false";
                let start_time = std::time::Instant::now();

                let cwd = {
                    let vars = state.shell_vars.lock().unwrap();
                    vars.get("PWD").cloned().unwrap_or_else(|| ".".to_string())
                };

                run_line_with(&mut state, &expanded_line, |prompt| rl.readline(prompt).ok());

                state.history_mgr.add_entry(line, state.last_exit_status, &cwd);
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

    // Sync $PWD with actual CWD before loading .jshrc or running any commands.
    sync_pwd();

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

    #[test]
    fn test_sync_pwd_updates_pwd_env_not_cwd() {
        let original_cwd = std::env::current_dir().unwrap();
        let original_pwd = std::env::var("PWD").ok();
        let fake_pwd = if original_cwd != std::path::Path::new("/tmp") {
            "/tmp"
        } else {
            "/"
        };
        unsafe {
            std::env::set_var("PWD", fake_pwd);
        }
        sync_pwd();
        assert_eq!(std::env::current_dir().unwrap(), original_cwd);
        assert_eq!(
            std::env::var("PWD").unwrap(),
            original_cwd.to_string_lossy().as_ref()
        );
        if let Some(pwd) = original_pwd {
            unsafe { std::env::set_var("PWD", &pwd); }
        }
    }

    #[test]
    fn test_cmd_option_execution() {
        let mut state = ShellState::new();
        assert!(!state.is_interactive);
        run_line_with(&mut state, "true", |_| None);
        assert_eq!(state.last_exit_status, 0);
    }
}
