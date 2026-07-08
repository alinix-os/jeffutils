/// Returns `true` if `name` is one of this shell's built-ins.
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "cd"
            | "pwd"
            | "echo"
            | "exit"
            | "export"
            | "unset"
            | "env"
            | "true"
            | "false"
            | ":"
            | "alias"
            | "unalias"
            | "set"
            | "help"
    )
}

use crate::shell::Shell;
use std::io::Write;

/// Dispatch a builtin by name. Returns `None` if `name` is not a builtin.
pub fn run(
    shell: &mut Shell,
    name: &str,
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Option<i32> {
    let status = match name {
        "cd" => cd(shell, args, err),
        "pwd" => pwd(out, err),
        "echo" => echo(args, out),
        "exit" => exit(shell, args, err),
        "export" => export(shell, args, out, err),
        "unset" => unset(shell, args, err),
        "env" => env(shell, out, err),
        "true" => 0,
        "false" => 1,
        ":" => 0,
        "alias" => alias(shell, args, out, err),
        "unalias" => unalias(shell, args, err),
        "set" => set(shell, out),
        "help" => help(out),
        _ => return None,
    };
    Some(status)
}

fn cd(shell: &mut Shell, args: &[String], err: &mut dyn Write) -> i32 {
    let target = match args.first().map(|s| s.as_str()) {
        None | Some("") => shell.var("HOME").unwrap_or_else(|| "/".to_string()),
        Some("-") => match shell.var("OLDPWD") {
            Some(d) => d,
            None => {
                let _ = writeln!(err, "cd: OLDPWD not set");
                return 1;
            }
        },
        Some(t) => t.to_string(),
    };

    let old = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    match std::env::set_current_dir(&target) {
        Ok(()) => {
            let new = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(target);
            shell.set_var("OLDPWD", &old);
            shell.set_var("PWD", &new);
            0
        }
        Err(e) => {
            let _ = writeln!(err, "cd: {target}: {e}");
            1
        }
    }
}

fn pwd(out: &mut dyn Write, _err: &mut dyn Write) -> i32 {
    match std::env::current_dir() {
        Ok(p) => {
            let _ = writeln!(out, "{}", p.display());
            0
        }
        Err(e) => {
            let _ = writeln!(out, "pwd: {e}");
            1
        }
    }
}

fn echo(args: &[String], out: &mut dyn Write) -> i32 {
    let mut i = 0;
    let mut newline = true;
    while i < args.len() {
        if args[i] == "-n" {
            newline = false;
            i += 1;
        } else {
            break;
        }
    }
    let rest = &args[i..];
    let line = rest.join(" ");
    if newline {
        let _ = writeln!(out, "{line}");
    } else {
        let _ = write!(out, "{line}");
    }
    0
}

fn exit(shell: &mut Shell, args: &[String], err: &mut dyn Write) -> i32 {
    let code = if let Some(a) = args.first() {
        match a.parse::<i32>() {
            Ok(c) => c,
            Err(_) => {
                let _ = writeln!(err, "exit: numeric argument required");
                2
            }
        }
    } else {
        shell.last_status
    };
    shell.should_exit = true;
    shell.exit_code = code;
    0
}

fn is_valid_ident(s: &str) -> bool {
    let mut ch = s.chars();
    match ch.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn export(shell: &mut Shell, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    if args.is_empty() {
        let mut names: Vec<&String> = shell.exported.iter().collect();
        names.sort();
        for name in names {
            if let Some(v) = shell.var(name) {
                let _ = writeln!(out, "export {name}={v}");
            } else {
                let _ = writeln!(out, "export {name}");
            }
        }
        return 0;
    }
    let mut rc = 0;
    for a in args {
        if let Some((name, value)) = a.split_once('=') {
            if name.is_empty() || !is_valid_ident(name) {
                let _ = writeln!(err, "export: '{a}': not a valid identifier");
                rc = 1;
                continue;
            }
            shell.set_var(name, value);
            shell.mark_exported(name);
        } else if is_valid_ident(a) {
            shell.mark_exported(a);
        } else {
            let _ = writeln!(err, "export: '{a}': not a valid identifier");
            rc = 1;
        }
    }
    rc
}

fn unset(shell: &mut Shell, args: &[String], err: &mut dyn Write) -> i32 {
    let mut rc = 0;
    for a in args {
        if is_valid_ident(a) {
            shell.unset_var(a);
        } else {
            let _ = writeln!(err, "unset: '{a}': not a valid identifier");
            rc = 1;
        }
    }
    rc
}

fn env(shell: &mut Shell, out: &mut dyn Write, _err: &mut dyn Write) -> i32 {
    let mut pairs: Vec<(String, String)> = shell.combined_env().into_iter().collect();
    pairs.sort();
    for (k, v) in pairs {
        let _ = writeln!(out, "{k}={v}");
    }
    0
}

fn alias(shell: &mut Shell, args: &[String], out: &mut dyn Write, _err: &mut dyn Write) -> i32 {
    if args.is_empty() {
        let mut names: Vec<&String> = shell.aliases.keys().collect();
        names.sort();
        for name in names {
            let val = &shell.aliases[name];
            let _ = writeln!(out, "{name}='{val}'");
        }
        return 0;
    }
    for a in args {
        if let Some((name, value)) = a.split_once('=') {
            shell.aliases.insert(name.to_string(), value.to_string());
        } else if let Some(v) = shell.aliases.get(a) {
            let _ = writeln!(out, "{a}='{v}'");
        } else {
            let _ = writeln!(out, "alias: {a}: not found");
        }
    }
    0
}

fn unalias(shell: &mut Shell, args: &[String], err: &mut dyn Write) -> i32 {
    let mut rc = 0;
    for a in args {
        if shell.aliases.remove(a).is_none() {
            let _ = writeln!(err, "unalias: {a}: not found");
            rc = 1;
        }
    }
    rc
}

fn set(shell: &mut Shell, out: &mut dyn Write) -> i32 {
    let mut pairs: Vec<(&String, &String)> = shell.vars.iter().collect();
    pairs.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in pairs {
        let _ = writeln!(out, "{k}={v}");
    }
    0
}

fn help(out: &mut dyn Write) -> i32 {
    let _ = writeln!(
        out,
        "Built-ins: cd pwd echo exit export unset env true false : alias unalias set help"
    );
    0
}

