use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::io::OwnedFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use crate::parser::lexer::RedirectTarget;
use crate::parser::{ExpandedCommand, ExpandedPipeline};
use crate::utils::expand_target;

/// Opens (creating if needed) the file used by an output redirection (`>`, `>>`).
fn open_output_file(path: &str, append: bool) -> File {
    let path = expand_target(path);
    let mut opts = OpenOptions::new();
    opts.write(true).create(true);
    if append {
        opts.append(true);
    } else {
        opts.truncate(true);
    }

    match opts.open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("jsh: {}: {}", path, e);
            OpenOptions::new()
                .write(true)
                .open("/dev/null")
                .unwrap_or_else(|_| panic!("jsh: falha ao abrir /dev/null"))
        }
    }
}

/// Opens the file used by an input redirection (`<`).
fn open_input_file(path: &str) -> File {
    let path = expand_target(path);
    match OpenOptions::new().read(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("jsh: {}: {}", path, e);
            OpenOptions::new()
                .read(true)
                .open("/dev/null")
                .unwrap_or_else(|_| panic!("jsh: falha ao abrir /dev/null"))
        }
    }
}

/// Duplicates an existing file descriptor (used by `2>&1`, `0<&3`, ...).
///
/// Best-effort on Unix: re-opens the descriptor through `/proc/self/fd`.
/// `writable` selects read vs write mode for the duplicate.
fn dup_fd(fd: i32, writable: bool) -> Stdio {
    let mut opts = OpenOptions::new();
    if writable {
        opts.write(true);
    } else {
        opts.read(true);
    }
    match opts.open(format!("/proc/self/fd/{}", fd)) {
        Ok(f) => Stdio::from(f),
        Err(_) => Stdio::inherit(),
    }
}

/// Builds a `Stdio` that feeds `content` to a child's stdin via a pipe.
fn string_to_stdio(content: &str) -> Stdio {
    let (read, mut write) = UnixStream::pair().unwrap();
    let _ = write.write_all(content.as_bytes());
    drop(write);
    Stdio::from(OwnedFd::from(read))
}

/// Builds the stdin/stdout/stderr `Stdio` for one command in a pipeline.
/// `capture_stdout` routes this command's stdout into a captured pipe
/// (`Command::stdout(Stdio::piped())`) instead of inheriting it, used for
/// the last stage of `$(...)` command substitution.
fn spawn_one(
    cmd: &ExpandedCommand,
    piped: bool,
    next_stdin: &mut Option<Stdio>,
    capture_stdout: bool,
) -> Command {
    let mut process = Command::new(&cmd.program);
    process.args(&cmd.args);
    for (k, v) in &cmd.env_vars {
        process.env(k, v);
    }

    let mut stdin_r = None;
    let mut stdout_r = None;
    let mut stderr_r = None;
    for r in &cmd.redirects {
        match r.fd {
            0 => stdin_r = Some(r),
            1 => stdout_r = Some(r),
            2 => stderr_r = Some(r),
            _ => {}
        }
    }

    // ---- stdin ----
    let stdin = if let Some(r) = stdin_r {
        match &r.target {
            RedirectTarget::File(p) => Stdio::from(open_input_file(p)),
            RedirectTarget::Fd(fd) => dup_fd(*fd, false),
            RedirectTarget::HereString(s) => string_to_stdio(&format!("{}\n", s)),
            RedirectTarget::Heredoc(_) => match &cmd.heredoc {
                Some(body) => string_to_stdio(body),
                None => Stdio::inherit(),
            },
        }
    } else if let Some(s) = next_stdin.take() {
        s
    } else {
        Stdio::inherit()
    };
    process.stdin(stdin);

    // ---- stdout ----
    let (stdout, pipe_write) = if let Some(r) = stdout_r {
        match &r.target {
            RedirectTarget::File(p) => (Stdio::from(open_output_file(p, r.append)), None),
            RedirectTarget::Fd(fd) => (dup_fd(*fd, true), None),
            _ => (Stdio::inherit(), None),
        }
    } else if piped {
        // Build a manual pipe so stderr can be merged into it (e.g. `2>&1`).
        let (read_end, write_end) = UnixStream::pair().unwrap();
        *next_stdin = Some(Stdio::from(OwnedFd::from(read_end)));
        (
            Stdio::from(OwnedFd::from(write_end.try_clone().unwrap())),
            Some(write_end),
        )
    } else if capture_stdout {
        (Stdio::piped(), None)
    } else {
        (Stdio::inherit(), None)
    };
    process.stdout(stdout);

    // ---- stderr ----
    let stderr = if let Some(r) = stderr_r {
        match &r.target {
            RedirectTarget::File(p) => Stdio::from(open_output_file(p, r.append)),
            RedirectTarget::Fd(target_fd) => {
                if *target_fd == 1 && pipe_write.is_some() {
                    // `2>&1` inside a pipeline: join the stdout pipe.
                    Stdio::from(OwnedFd::from(pipe_write.as_ref().unwrap().try_clone().unwrap()))
                } else {
                    dup_fd(*target_fd, true)
                }
            }
            _ => Stdio::inherit(),
        }
    } else {
        Stdio::inherit()
    };
    process.stderr(stderr);

    process
}

pub fn execute_with(pipe: ExpandedPipeline, state: &crate::shell::ShellState) -> i32 {
    let quiet = state.quiet_errors;
    let n = pipe.commands.len();
    if n == 0 {
        return 0;
    }

    // Block SIGINT in the shell while children run so Ctrl+C kills only
    // the foreground process group, not the shell itself.
    let is_tty = unsafe { libc::isatty(libc::STDIN_FILENO) != 0 };
    let mut old_sigint: usize = 0;
    if is_tty {
        unsafe {
            old_sigint = libc::signal(libc::SIGINT, libc::SIG_IGN);
        }
    }

    let mut children = Vec::new();
    let mut next_stdin: Option<Stdio> = None;
    let mut pgid = 0;

    for i in 0..n {
        let cmd = &pipe.commands[i];
        let piped = i < n - 1;
        let mut process = spawn_one(cmd, piped, &mut next_stdin, false);

        unsafe {
            if i == 0 {
                process.pre_exec(|| {
                    let _ = libc::setpgid(0, 0);
                    libc::signal(libc::SIGTTOU, libc::SIG_DFL);
                    libc::signal(libc::SIGTTIN, libc::SIG_DFL);
                    libc::signal(libc::SIGTSTP, libc::SIG_DFL);
                    libc::signal(libc::SIGQUIT, libc::SIG_DFL);
                    Ok(())
                });
            } else {
                let first_pgid = pgid;
                process.pre_exec(move || {
                    let _ = libc::setpgid(0, first_pgid);
                    libc::signal(libc::SIGTTOU, libc::SIG_DFL);
                    libc::signal(libc::SIGTTIN, libc::SIG_DFL);
                    libc::signal(libc::SIGTSTP, libc::SIG_DFL);
                    libc::signal(libc::SIGQUIT, libc::SIG_DFL);
                    Ok(())
                });
            }
        }

        match process.spawn() {
            Ok(child) => {
                let child_id = child.id() as libc::pid_t;
                if i == 0 {
                    pgid = child_id;
                }
                unsafe {
                    let target = if i == 0 { child_id } else { pgid };
                    let _ = libc::setpgid(child_id, target);
                }
                children.push(child);
            }
            Err(e) => {
                if !quiet {
                    eprintln!("jsh: {}: {}", cmd.program, e);
                    if e.kind() == std::io::ErrorKind::NotFound {
                        if let Some(suggestion) = crate::utils::suggest_command(&cmd.program, state) {
                            eprintln!("Você quis dizer '{}'?", suggestion);
                        }
                    }
                }
                if is_tty {
                    unsafe {
                        libc::tcsetpgrp(libc::STDIN_FILENO, libc::getpgrp());
                        libc::signal(libc::SIGINT, old_sigint);
                    }
                }
                return 127;
            }
        }
    }

    if is_tty && pgid != 0 {
        unsafe {
            libc::tcsetpgrp(libc::STDIN_FILENO, pgid);
        }
    }

    let mut last_status = 0;
    for mut child in children {
        match child.wait() {
            Ok(status) => last_status = status.code().unwrap_or(0),
            Err(_) => last_status = 1,
        }
    }

    if is_tty {
        unsafe {
            libc::tcsetpgrp(libc::STDIN_FILENO, libc::getpgrp());
            libc::signal(libc::SIGINT, old_sigint);
        }
    }

    last_status
}

/// Spawns a pipeline in the background without waiting for it (`cmd &`).
/// Returns the PID of the last stage, if it started successfully. Not full
/// job control (no `jobs`/`fg`/`bg`) — just fire-and-forget, like a
/// disowned background job.
pub fn spawn_detached(pipe: ExpandedPipeline) -> Option<u32> {
    let n = pipe.commands.len();
    if n == 0 {
        return None;
    }
    let mut next_stdin: Option<Stdio> = None;
    let mut last_pid = None;

    for i in 0..n {
        let cmd = &pipe.commands[i];
        let piped = i < n - 1;
        let mut process = spawn_one(cmd, piped, &mut next_stdin, false);
        if next_stdin.is_none() && i == 0 {
            process.stdin(Stdio::null());
        }
        match process.spawn() {
            Ok(child) => last_pid = Some(child.id()),
            Err(e) => {
                eprintln!("jsh: {}: {}", cmd.program, e);
                return None;
            }
        }
    }

    last_pid
}

/// Like `execute`, but captures the final command's stdout and returns it
/// instead of printing it — used for `$(...)` command substitution.
pub fn execute_capture(pipe: ExpandedPipeline) -> Vec<u8> {
    let n = pipe.commands.len();
    if n == 0 {
        return Vec::new();
    }
    let mut children = Vec::new();
    let mut next_stdin: Option<Stdio> = None;

    for i in 0..n {
        let cmd = &pipe.commands[i];
        let piped = i < n - 1;
        let is_last = i == n - 1;
        let mut process = spawn_one(cmd, piped, &mut next_stdin, is_last);

        match process.spawn() {
            Ok(child) => children.push(child),
            Err(e) => {
                eprintln!("jsh: {}: {}", cmd.program, e);
                return Vec::new();
            }
        }
    }

    let mut last_child = children.pop();
    let mut output = Vec::new();
    if let Some(child) = last_child.as_mut() {
        if let Some(mut stdout) = child.stdout.take() {
            use std::io::Read;
            let _ = stdout.read_to_end(&mut output);
        }
    }

    for mut child in children {
        let _ = child.wait();
    }
    if let Some(mut child) = last_child {
        let _ = child.wait();
    }

    output
}
