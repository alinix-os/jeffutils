pub mod pipeline;

use crate::parser::{ExpandedPipeline, ListOp};
use crate::shell::ShellState;

/// Runs a fully parsed `CommandList` (a line possibly containing `;`, `&&`,
/// `||`, and multiple pipelines), applying short-circuit semantics based on
/// each pipeline's exit status. Builtins are dispatched in-process via
/// `handle_builtin` so `cd /tmp && ls` etc. behave correctly.
pub fn run_command_list(
    state: &mut ShellState,
    list: &crate::parser::CommandList,
    heredoc_bodies: &[Option<String>],
) {
    let mut prev_op: Option<ListOp> = None;

    for (i, (andor, op)) in list.items.iter().enumerate() {
        let should_run = match prev_op {
            None => true,
            Some(ListOp::Seq) => true,
            Some(ListOp::And) => state.last_exit_status == 0,
            Some(ListOp::Or) => state.last_exit_status != 0,
        };

        if should_run {
            let heredoc = heredoc_bodies.get(i).and_then(|o| o.as_deref());
            run_and_or(state, andor, heredoc);
        }

        prev_op = *op;
    }
}

fn run_and_or(state: &mut ShellState, andor: &crate::parser::AndOrList, heredoc: Option<&str>) {
    // Assignment-only pipeline, e.g. `FOO=bar` with no command: set the var
    // and don't spawn anything.
    if andor.pipeline.commands.len() == 1 {
        let cmd = &andor.pipeline.commands[0];
        if cmd.args.is_empty() && cmd.redirects.is_empty() {
            if !cmd.env_vars.is_empty() && (cmd.program.segments.is_empty() || ShellState::as_assignment(&cmd.program).is_some()) {
                for (name, value) in &cmd.env_vars {
                    let expanded_value = state.expand_word_single(&crate::parser::Word::literal(value));
                    state.set_var(name, &expanded_value);
                }
                state.last_exit_status = 0;
                return;
            }
            if let Some((name, value)) = ShellState::as_assignment(&cmd.program) {
                let expanded_value = state.expand_word_single(&crate::parser::Word::literal(value));
                state.set_var(&name, &expanded_value);
                state.last_exit_status = 0;
                return;
            }
        }
    }

    let mut expanded: ExpandedPipeline = state.expand_pipeline(&andor.pipeline, heredoc);
    if expanded.commands.is_empty() {
        return;
    }

    let is_time = expanded.commands[0].program == "time";
    if is_time {
        if expanded.commands[0].args.is_empty() {
            let sep = get_decimal_separator();
            eprintln!("real    0m0{}000s", sep);
            eprintln!("user    0m0{}000s", sep);
            eprintln!("sys     0m0{}000s", sep);
            state.last_exit_status = 0;
            return;
        }

        // Shift arguments to strip "time"
        let sub_program = expanded.commands[0].args[0].clone();
        let sub_args = expanded.commands[0].args[1..].to_vec();
        expanded.commands[0].program = sub_program;
        expanded.commands[0].args = sub_args;

        let mut usage_before = unsafe { std::mem::zeroed::<libc::rusage>() };
        unsafe {
            libc::getrusage(libc::RUSAGE_CHILDREN, &mut usage_before);
        }
        let start_time = std::time::Instant::now();

        execute_expanded(state, expanded, andor.background);

        let elapsed = start_time.elapsed().as_secs_f64();
        let mut usage_after = unsafe { std::mem::zeroed::<libc::rusage>() };
        unsafe {
            libc::getrusage(libc::RUSAGE_CHILDREN, &mut usage_after);
        }

        let user_time = (usage_after.ru_utime.tv_sec - usage_before.ru_utime.tv_sec) as f64
            + (usage_after.ru_utime.tv_usec - usage_before.ru_utime.tv_usec) as f64 / 1_000_000.0;
        let sys_time = (usage_after.ru_stime.tv_sec - usage_before.ru_stime.tv_sec) as f64
            + (usage_after.ru_stime.tv_usec - usage_before.ru_stime.tv_usec) as f64 / 1_000_000.0;

        eprintln!("real    {}", format_time(elapsed));
        eprintln!("user    {}", format_time(user_time));
        eprintln!("sys     {}", format_time(sys_time));
    } else {
        execute_expanded(state, expanded, andor.background);
    }
}

fn execute_expanded(state: &mut ShellState, expanded: ExpandedPipeline, background: bool) {
    // Intercept `exec` builtin command to replace current process
    if expanded.commands.len() == 1 && expanded.commands[0].program == "exec" {
        let cmd = &expanded.commands[0];
        if cmd.args.is_empty() {
            // exec with no command is used to apply redirections to the shell.
            apply_current_redirects(&cmd.redirects, cmd.heredoc.as_deref());
            state.last_exit_status = 0;
            return;
        }

        // Apply redirections in-process
        apply_current_redirects(&cmd.redirects, cmd.heredoc.as_deref());

        let target_cmd = &cmd.args[0];
        let target_args = &cmd.args[1..];

        use std::os::unix::process::CommandExt;
        let mut process = std::process::Command::new(target_cmd);
        process.args(target_args);
        for (k, v) in &cmd.env_vars {
            process.env(k, v);
        }

        let err = process.exec();
        eprintln!("jsh: exec: {}: {}", target_cmd, err);
        let exit_code = match err.kind() {
            std::io::ErrorKind::NotFound => 127,
            std::io::ErrorKind::PermissionDenied => 126,
            _ => 1,
        };
        std::process::exit(exit_code);
    }

    // Single command, no redirects/pipe: try builtins first (in-process).
    if expanded.commands.len() == 1 && expanded.commands[0].redirects.is_empty() {
        let cmd = &expanded.commands[0];
        let mut argv = vec![cmd.program.clone()];
        argv.extend(cmd.args.clone());

        let prev_vars: Vec<(String, Option<String>)> = cmd
            .env_vars
            .iter()
            .map(|(k, _)| (k.clone(), state.shell_vars.lock().unwrap().get(k).cloned()))
            .collect();
        for (k, v) in &cmd.env_vars {
            state.set_var(k, v);
        }

        if let Some(status) = crate::builtin::handle_builtin(&argv, state) {
            for (k, old_v) in prev_vars {
                if let Some(val) = old_v {
                    state.set_var(&k, &val);
                } else {
                    state.unset_var(&k);
                }
            }
            state.last_exit_status = status;
            return;
        }

        // User-defined shell functions win over external programs of the
        // same name (e.g. a `proj()` shortcut should shadow /usr/bin/proj).
        if state.functions.lock().unwrap().contains_key(&cmd.program) {
            let status = state.call_function(&cmd.program, &cmd.args);
            for (k, old_v) in prev_vars {
                if let Some(val) = old_v {
                    state.set_var(&k, &val);
                } else {
                    state.unset_var(&k);
                }
            }
            state.last_exit_status = status;
            return;
        }

        // Auto-cd feature: if command is exactly the name of a directory and has no args, cd into it
        // Only trigger if the command is not a valid executable (so `clear` runs the command, not cd into `clear/`)
        if cmd.args.is_empty() && !crate::builtin::is_executable(&cmd.program) {
            let path = std::path::Path::new(&cmd.program);
            if path.is_dir() {
                let argv = vec!["cd".to_string(), cmd.program.clone()];
                if let Some(status) = crate::builtin::handle_builtin(&argv, state) {
                    state.last_exit_status = status;
                    return;
                }
            }
        }

        // Not a builtin and not on PATH: if `.jshrc` sourced real bash
        // scripts (e.g. nvm.sh), retry the command through bash so
        // functions defined there (`nvm`, ...) still work.
        if !crate::builtin::is_executable(&cmd.program) {
            if let Some(status) = state.try_bash_fallback(&cmd.program, &cmd.args) {
                state.last_exit_status = status;
                return;
            }
        }
    }

    if background {
        let pid = pipeline::spawn_detached(expanded);
        if let Some(pid) = pid {
            eprintln!("[bg] {}", pid);
        }
        state.last_exit_status = 0;
        return;
    }

    state.last_exit_status = pipeline::execute_with(expanded, state);
}

fn get_decimal_separator() -> &'static str {
    for var in &["LC_NUMERIC", "LC_ALL", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            let val_lower = val.to_lowercase();
            if val_lower.starts_with("pt")
                || val_lower.starts_with("fr")
                || val_lower.starts_with("de")
                || val_lower.starts_with("es")
                || val_lower.starts_with("it")
                || val_lower.starts_with("ru")
                || val_lower.starts_with("nl")
                || val_lower.starts_with("da")
                || val_lower.starts_with("sv")
                || val_lower.starts_with("nb")
                || val_lower.starts_with("nn")
                || val_lower.starts_with("fi")
                || val_lower.starts_with("pl")
                || val_lower.starts_with("cs")
                || val_lower.starts_with("sk")
                || val_lower.starts_with("hu")
                || val_lower.starts_with("tr")
                || val_lower.starts_with("el")
            {
                return ",";
            }
        }
    }
    "."
}

fn format_time(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as u64;
    let remaining_seconds = seconds - (minutes as f64 * 60.0);
    let sec_str = format!("{:.3}", remaining_seconds);
    let sep = get_decimal_separator();
    let formatted_secs = if sep == "," {
        sec_str.replace('.', ",")
    } else {
        sec_str
    };
    format!("{}m{}s", minutes, formatted_secs)
}

fn apply_current_redirects(redirects: &[crate::parser::lexer::Redirect], heredoc: Option<&str>) {
    use crate::parser::lexer::RedirectTarget;
    use std::os::unix::io::AsRawFd;

    unsafe extern "C" {
        fn dup2(oldfd: std::os::raw::c_int, newfd: std::os::raw::c_int) -> std::os::raw::c_int;
    }

    for r in redirects {
        let target_fd = r.fd;

        match &r.target {
            RedirectTarget::File(path) => {
                let path = crate::utils::expand_target(path);
                if target_fd == 0 {
                    // Input redirection
                    if let Ok(file) = std::fs::OpenOptions::new().read(true).open(&path) {
                        unsafe {
                            dup2(file.as_raw_fd(), 0);
                        }
                    } else {
                        eprintln!("jsh: {}: Arquivo não encontrado", path);
                        std::process::exit(1);
                    }
                } else {
                    // Output redirection
                    let mut opts = std::fs::OpenOptions::new();
                    opts.write(true).create(true);
                    if r.append {
                        opts.append(true);
                    } else {
                        opts.truncate(true);
                    }
                    if let Ok(file) = opts.open(&path) {
                        let fd = file.as_raw_fd();
                        if target_fd == -1 {
                            // &> redirects both stdout and stderr
                            unsafe {
                                dup2(fd, 1);
                                dup2(fd, 2);
                            }
                        } else {
                            unsafe {
                                dup2(fd, target_fd);
                            }
                        }
                    } else {
                        eprintln!("jsh: {}: Erro ao abrir arquivo", path);
                        std::process::exit(1);
                    }
                }
            }
            RedirectTarget::Fd(source_fd) => {
                let fd_to_dup = if *source_fd == 1 && target_fd == 2 {
                    1
                } else if *source_fd == 2 && target_fd == 1 {
                    2
                } else {
                    *source_fd
                };
                unsafe {
                    if target_fd == -1 {
                        dup2(fd_to_dup, 1);
                        dup2(fd_to_dup, 2);
                    } else {
                        dup2(fd_to_dup, target_fd);
                    }
                }
            }
            RedirectTarget::HereString(s) => {
                if target_fd == 0 || target_fd == -1 {
                    use std::io::Write;
                    if let Ok((read, mut write)) = std::os::unix::net::UnixStream::pair() {
                        let _ = write.write_all(format!("{}\n", s).as_bytes());
                        drop(write);
                        unsafe {
                            dup2(read.as_raw_fd(), 0);
                        }
                    }
                }
            }
            RedirectTarget::Heredoc(_) => {
                if let Some(body) = heredoc {
                    if target_fd == 0 || target_fd == -1 {
                        use std::io::Write;
                        if let Ok((read, mut write)) = std::os::unix::net::UnixStream::pair() {
                            let _ = write.write_all(body.as_bytes());
                            drop(write);
                            unsafe {
                                dup2(read.as_raw_fd(), 0);
                            }
                        }
                    }
                }
            }
        }
    }
}
