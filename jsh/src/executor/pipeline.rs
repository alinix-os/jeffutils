use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::io::OwnedFd;
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};

use crate::parser::lexer::RedirectTarget;
use crate::parser::Pipeline;
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

pub fn execute(pipe: Pipeline) -> i32 {
    let n = pipe.commands.len();
    let mut children = Vec::new();
    let mut next_stdin: Option<Stdio> = None;

    for i in 0..n {
        let cmd = &pipe.commands[i];
        let mut process = Command::new(&cmd.program);
        process.args(&cmd.args);

        // Classify redirects by file descriptor.
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

        let piped = i < n - 1;

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
            next_stdin = Some(Stdio::from(OwnedFd::from(read_end)));
            (
                Stdio::from(OwnedFd::from(write_end.try_clone().unwrap())),
                Some(write_end),
            )
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
                        Stdio::from(OwnedFd::from(pipe_write.unwrap().try_clone().unwrap()))
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

        match process.spawn() {
            Ok(child) => children.push(child),
            Err(e) => {
                eprintln!("jsh: {}: {}", cmd.program, e);
                return 127;
            }
        }
    }

    let mut last_status = 0;
    for mut child in children {
        match child.wait() {
            Ok(status) => last_status = status.code().unwrap_or(0),
            Err(_) => last_status = 1,
        }
    }

    last_status
}
