//! Command executor: runs the AST.
//!
//! Supports pipelines, redirections, background jobs, command-scoped
//! assignments and builtins. Builtins that appear inside a pipeline are
//! executed in-process and their output is fed to the next stage through a
//! Unix-domain socket pair, which keeps the implementation dependency-free.

use crate::ast::{Redir, SimpleCommand};
use crate::builtins;
use crate::lexer;
use crate::shell::Shell;
use std::fs::{File, OpenOptions};
use std::io::{self, Cursor, Write};
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};

/// A command whose words have already been expanded against the current
/// shell state (used at execution time so that earlier assignments in the
/// same line are visible).
struct Expanded {
    argv: Vec<String>,
    redirs: Vec<Redir>,
    assigns: Vec<(String, String)>,
}

/// Expand a raw [`SimpleCommand`] into an [`Expanded`] command using the
fn expand_command(shell: &Shell, cmd: &SimpleCommand) -> Result<Expanded, String> {
    let lookup: &lexer::Lookup = &|name: &str| shell.var(name);
    let mut argv = Vec::new();
    for w in &cmd.argv {
        argv.extend(lexer::expand_word(w, lookup, shell.last_status)?);
    }
    let mut redirs = Vec::new();
    for r in &cmd.redirs {
        match r {
            Redir::In(p) => redirs.push(Redir::In(lexer::expand_one(p, lookup, shell.last_status)?)),
            Redir::Out(p, a) => {
                redirs.push(Redir::Out(lexer::expand_one(p, lookup, shell.last_status)?, *a))
            }
            Redir::Err(p, a) => {
                redirs.push(Redir::Err(lexer::expand_one(p, lookup, shell.last_status)?, *a))
            }
            Redir::Both(p) => {
                redirs.push(Redir::Both(lexer::expand_one(p, lookup, shell.last_status)?))
            }
            Redir::Dup(f, t) => redirs.push(Redir::Dup(*f, *t)),
        }
    }
    let mut assigns = Vec::new();
    for (k, v) in &cmd.assigns {
        assigns.push((k.clone(), lexer::expand_one(v, lookup, shell.last_status)?));
    }
    Ok(Expanded {
        argv,
        redirs,
        assigns,
    })
}


fn open_file(path: &str, append: bool) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(path)
}

/// Duplicate an existing file descriptor into a fresh [`OwnedFd`].
///
/// `from_raw_fd` would take ownership and close the original on drop, so we
/// `forget` the temporary handle after cloning.
fn dup_fd(fd: i32) -> io::Result<OwnedFd> {
    let f = unsafe { File::from_raw_fd(fd) };
    let dup = f.try_clone()?;
    std::mem::forget(f);
    Ok(OwnedFd::from(dup))
}

/// Build and spawn a single external command, optionally wiring its stdin to
/// `stdin` and creating an output pipe for the next stage.
fn spawn_external(
    shell: &Shell,
    cmd: &Expanded,
    stdin: Option<Stdio>,
    make_pipe: bool,
) -> io::Result<(Child, Option<Stdio>)> {
    let program = &cmd.argv[0];
    let mut command = Command::new(program);
    command.args(&cmd.argv[1..]);

    let env = shell.combined_env();
    command.env_clear();
    command.envs(env);
    for (k, v) in &cmd.assigns {
        command.env(k, v);
    }

    // Resolve redirections into concrete file descriptors held in `slot`,
    // applying them left-to-right so that e.g. `2>file 1>&2` duplicates the
    // already-redirected fd 2. (No libc needed: we clone `OwnedFd`s.)
    let mut slot: [Option<OwnedFd>; 3] = [None, None, None];
    let mut next_prev: Option<Stdio> = None;
    if make_pipe {
        let (r, w) = UnixStream::pair()?;
        next_prev = Some(Stdio::from(OwnedFd::from(r)));
        slot[1] = Some(OwnedFd::from(w));
    }

    for r in &cmd.redirs {
        match r {
            Redir::In(p) => {
                let f = File::open(p)?;
                slot[0] = Some(OwnedFd::from(f));
            }
            Redir::Out(p, append) => {
                let f = open_file(p, *append)?;
                slot[1] = Some(OwnedFd::from(f));
            }
            Redir::Err(p, append) => {
                let f = open_file(p, *append)?;
                slot[2] = Some(OwnedFd::from(f));
            }
            Redir::Both(p) => {
                let f = open_file(p, false)?;
                let fc = f.try_clone()?;
                slot[1] = Some(OwnedFd::from(f));
                slot[2] = Some(OwnedFd::from(fc));
            }
            Redir::Dup(fd, target) => {
                let newfd = if (*target as usize) < 3 {
                    match &slot[*target as usize] {
                        Some(o) => o.try_clone()?,
                        None => dup_fd(*target as i32)?,
                    }
                } else {
                    dup_fd(*target as i32)?
                };
                slot[*fd as usize] = Some(newfd);
            }
        }
    }

    if let Some(s) = slot[0].take() {
        command.stdin(Stdio::from(s));
    } else if let Some(s) = stdin {
        command.stdin(s);
    }
    if let Some(s) = slot[1].take() {
        command.stdout(Stdio::from(s));
    }
    if let Some(s) = slot[2].take() {
        command.stderr(Stdio::from(s));
    }

    let child = command.spawn()?;
    Ok((child, next_prev))
}

/// Writers for a standalone builtin, honouring `>`, `>>`, `2>`, `&>` and
/// fd duplication (`2>&1`, `1>&2`) using the same left-to-right slot logic as
/// external commands.
fn builtin_writers(cmd: &Expanded) -> (Box<dyn Write>, Box<dyn Write>) {
    let mut slot: [Option<OwnedFd>; 3] = [None, None, None];
    for r in &cmd.redirs {
        match r {
            Redir::In(_) => {}
            Redir::Out(p, append) => {
                if let Ok(f) = open_file(p, *append) {
                    slot[1] = Some(OwnedFd::from(f));
                }
            }
            Redir::Err(p, append) => {
                if let Ok(f) = open_file(p, *append) {
                    slot[2] = Some(OwnedFd::from(f));
                }
            }
            Redir::Both(p) => {
                if let Ok(f) = open_file(p, false) {
                    if let Ok(fc) = f.try_clone() {
                        slot[1] = Some(OwnedFd::from(f));
                        slot[2] = Some(OwnedFd::from(fc));
                    }
                }
            }
            Redir::Dup(fd, target) => {
                let newfd = if (*target as usize) < 3 {
                    match &slot[*target as usize] {
                        Some(o) => o.try_clone().ok(),
                        None => dup_fd(*target as i32).ok(),
                    }
                } else {
                    dup_fd(*target as i32).ok()
                };
                if let Some(d) = newfd {
                    slot[*fd as usize] = Some(d);
                }
            }
        }
    }
    let out: Box<dyn Write> = match slot[1].take() {
        Some(o) => Box::new(File::from(o)),
        None => Box::new(io::stdout()),
    };
    let err: Box<dyn Write> = match slot[2].take() {
        Some(o) => Box::new(File::from(o)),
        None => Box::new(io::stderr()),
    };
    (out, err)
}

fn apply_assigns(shell: &mut Shell, cmd: &Expanded) -> Vec<(String, Option<String>)> {
    let saved: Vec<(String, Option<String>)> = cmd
        .assigns
        .iter()
        .map(|(k, _)| (k.clone(), shell.vars.get(k).cloned()))
        .collect();
    for (k, v) in &cmd.assigns {
        shell.set_var(k, v);
    }
    saved
}

fn restore_assigns(shell: &mut Shell, saved: &[(String, Option<String>)]) {
    for (k, old) in saved {
        match old {
            Some(v) => shell.set_var(k, v),
            None => shell.unset_var(k),
        }
    }
}

fn run_builtin_standalone(shell: &mut Shell, cmd: &Expanded) -> i32 {
    let saved = apply_assigns(shell, cmd);
    let (mut out, mut err) = builtin_writers(cmd);
    let status = builtins::run(shell, &cmd.argv[0], &cmd.argv[1..], &mut out, &mut err)
        .unwrap_or(127);
    let _ = out.flush();
    let _ = err.flush();
    restore_assigns(shell, &saved);
    status
}

fn run_builtin_capture(
    shell: &mut Shell,
    cmd: &Expanded,
    out_buf: &mut Vec<u8>,
    err_buf: &mut Vec<u8>,
) -> i32 {
    let saved = apply_assigns(shell, cmd);
    let mut out = Cursor::new(Vec::new());
    let mut err = Cursor::new(Vec::new());
    let status = builtins::run(shell, &cmd.argv[0], &cmd.argv[1..], &mut out, &mut err)
        .unwrap_or(127);
    restore_assigns(shell, &saved);
    *out_buf = out.into_inner();
    *err_buf = err.into_inner();
    status
}

/// Run a single command (no pipeline).
fn run_single(shell: &mut Shell, cmd: &SimpleCommand) -> i32 {
    let exp = match expand_command(shell, cmd) {
        Ok(e) => e,
        Err(msg) => {
            eprintln!("sh: {msg}");
            return 2;
        }
    };

    if exp.argv.is_empty() {
        for (k, v) in &exp.assigns {
            shell.set_var(k, v);
        }
        return 0;
    }

    let name = &exp.argv[0];
    if builtins::is_builtin(name) {
        return run_builtin_standalone(shell, &exp);
    }

    match spawn_external(shell, &exp, None, false) {
        Ok((mut child, _)) => child.wait().map(|s| s.code().unwrap_or(127)).unwrap_or(127),
        Err(e) => {
            eprintln!("sh: {name}: {e}");
            127
        }
    }
}

/// Run a multi-stage pipeline, returning the status of the last stage.
fn run_multi(shell: &mut Shell, pipeline: &crate::ast::Pipeline) -> i32 {
    let m = pipeline.commands.len();
    let mut prev: Option<Stdio> = None;
    let mut children: Vec<Child> = Vec::new();
    let mut last_status = 0;

    for (idx, cmd) in pipeline.commands.iter().enumerate() {
        let is_last = idx == m - 1;
        let exp = match expand_command(shell, cmd) {
            Ok(e) => e,
            Err(msg) => {
                eprintln!("sh: {msg}");
                last_status = 2;
                continue;
            }
        };

        if !exp.argv.is_empty() && builtins::is_builtin(&exp.argv[0]) {
            let mut out_buf = Vec::new();
            let mut err_buf = Vec::new();
            last_status = run_builtin_capture(shell, &exp, &mut out_buf, &mut err_buf);

            if !is_last {
                if let Ok((r, mut w)) = UnixStream::pair() {
                    let _ = w.write_all(&out_buf);
                    drop(w);
                    prev = Some(Stdio::from(OwnedFd::from(r)));
                }
            } else {
                write_builtin_output(&exp, &out_buf, &err_buf);
            }
            continue;
        }

        let make_pipe = !is_last;
        match spawn_external(shell, &exp, prev.take(), make_pipe) {
            Ok((child, next_prev)) => {
                prev = next_prev;
                children.push(child);
            }
            Err(e) => {
                eprintln!(
                    "sh: {}: {e}",
                    exp.argv.first().unwrap_or(&String::new())
                );
                last_status = 127;
            }
        }
    }

    for mut c in children {
        let _ = c.wait();
    }
    last_status
}


/// Write a captured builtin's output to the terminal or to redirection files.
fn write_builtin_output(cmd: &Expanded, out_buf: &[u8], err_buf: &[u8]) {
    for r in &cmd.redirs {
        match r {
            Redir::Out(p, append) => {
                if let Ok(mut f) = open_file(p, *append) {
                    let _ = f.write_all(out_buf);
                }
            }
            Redir::Both(p) => {
                if let Ok(mut f) = open_file(p, false) {
                    let _ = f.write_all(out_buf);
                }
            }
            Redir::Err(p, append) => {
                if let Ok(mut f) = open_file(p, *append) {
                    let _ = f.write_all(err_buf);
                }
            }
            _ => {}
        }
    }
    // Default destinations when no matching redirection was applied.
    if !cmd.redirs.iter().any(|r| matches!(r, Redir::Out(_, _) | Redir::Both(_))) {
        print!("{}", String::from_utf8_lossy(out_buf));
        let _ = io::stdout().flush();
    }
    if !cmd.redirs.iter().any(|r| matches!(r, Redir::Err(_, _) | Redir::Both(_))) {
        eprint!("{}", String::from_utf8_lossy(err_buf));
    }
}

/// Execute a pipeline (single or multi-stage).
pub fn run_pipeline(shell: &mut Shell, pipeline: &crate::ast::Pipeline) -> i32 {
    if pipeline.commands.len() == 1 {
        run_single(shell, &pipeline.commands[0])
    } else {
        run_multi(shell, pipeline)
    }
}

/// Run a parsed program, honouring `&&`, `||`, `;` and background jobs.
///
/// `&&`/`||` are evaluated left-to-right with short-circuit semantics: a job
/// is skipped when the preceding connector and status say so, but evaluation
/// continues so that e.g. `false && a || b` still runs `b`.
pub fn execute(shell: &mut Shell, program: &crate::ast::Program) -> i32 {
    use crate::ast::Connector;
    let mut status = 0;
    let mut prev_status = 0;
    let mut prev_connector = Connector::Last;

    for job in &program.jobs {
        if shell.should_exit {
            break;
        }
        // Decide whether this job runs, based on the previous connector.
        let run = match prev_connector {
            Connector::And => prev_status == 0,
            Connector::Or => prev_status != 0,
            _ => true,
        };

        if run {
            if job.background {
                spawn_background(shell, job);
                status = 0;
            } else {
                status = run_pipeline(shell, &job.pipeline);
                shell.last_status = status;
            }
        }

        prev_connector = job.connector;
        prev_status = status;
    }
    status
}

/// Launch a job in the background using a detached thread with an isolated
/// (cloned) shell state, mimicking subshell semantics for variable changes.
fn spawn_background(shell: &Shell, job: &crate::ast::Job) {
    let mut sh = shell.clone();
    let pipeline = job.pipeline.clone();
    std::thread::spawn(move || {
        run_pipeline(&mut sh, &pipeline);
    });
}

