//! Shared utility helpers for `jsh`.
//!
//! Cross-module helpers (string/path expansion) used by several subsystems.

use std::env;

/// Expands `$VAR` and `${VAR}` references using a caller-supplied lookup
/// function (shell vars + special vars like `$?`, falling back to env).
pub fn expand_env_vars_with(arg: &str, lookup: impl Fn(&str) -> String) -> String {
    let mut result = String::new();
    let mut chars = arg.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let mut var_name = String::new();
            let mut is_braced = false;
            if chars.peek() == Some(&'{') {
                chars.next();
                is_braced = true;
            }
            while let Some(&vc) = chars.peek() {
                if is_braced {
                    if vc == '}' {
                        chars.next();
                        break;
                    }
                    var_name.push(chars.next().unwrap());
                } else if vc.is_alphanumeric() || vc == '_' {
                    var_name.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            if !var_name.is_empty() {
                result.push_str(&lookup(&var_name));
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Expands `$VAR` and `${VAR}` references using the process environment.
pub fn expand_env_vars(arg: &str) -> String {
    expand_env_vars_with(arg, |name| env::var(name).unwrap_or_default())
}

/// Expands a leading `~` (or `~/`) to the given home directory string.
pub fn expand_tilde_with(path: &str, home: &str) -> String {
    if path == "~" {
        home.to_string()
    } else if let Some(rest) = path.strip_prefix("~/") {
        format!("{}/{}", home, rest)
    } else {
        path.to_string()
    }
}

pub fn suggest_command(cmd: &str, state: &crate::shell::ShellState) -> Option<String> {
    if std::env::var("JSH_DID_YOU_MEAN").unwrap_or_else(|_| "1".to_string()) == "0" {
        return None;
    }

    let builtins = [
        "cd", "export", "unset", "set", "alias", "unalias",
        "source", "true", "false", "exec", "exit",
    ];

    let mut best_match: Option<String> = None;
    let mut min_distance = 3;

    for cand in builtins {
        if cand == cmd { continue; }
        let dist = strsim::damerau_levenshtein(cmd, cand);
        if dist > 0 && dist < min_distance {
            min_distance = dist;
            best_match = Some(cand.to_string());
        }
    }

    if let Ok(aliases) = state.aliases.lock() {
        for name in aliases.keys() {
            if name == cmd { continue; }
            let dist = strsim::damerau_levenshtein(cmd, name.as_str());
            if dist > 0 && dist < min_distance {
                min_distance = dist;
                best_match = Some(name.clone());
            }
        }
    }
    if let Ok(funcs) = state.functions.lock() {
        for name in funcs.keys() {
            if name == cmd { continue; }
            let dist = strsim::damerau_levenshtein(cmd, name.as_str());
            if dist > 0 && dist < min_distance {
                min_distance = dist;
                best_match = Some(name.clone());
            }
        }
    }

    best_match
}

/// Expands a leading `~` (or `~/`) to the value of `$HOME`.
pub fn expand_tilde(path: &str) -> String {
    let home = env::var("HOME").unwrap_or_default();
    expand_tilde_with(path, &home)
}

/// Expands environment variables and tilde in a path (used for redirect targets).
pub fn expand_target(path: &str) -> String {
    expand_tilde(&expand_env_vars(path))
}

/// Emits OSC 7 (working directory) and OSC 0 (window/tab title) escape sequences
/// to inform terminal emulators (Ptyxis, GNOME Terminal, Konsole, Alacritty, Kitty)
/// of the current working directory so new tabs/windows (Ctrl+Shift+T/N) open in
/// the same directory.
pub fn emit_osc7() {
    use std::io::{IsTerminal, Write};

    // Only emit escape sequences if stdout is connected to a terminal.
    if !std::io::stdout().is_terminal() {
        return;
    }

    let pwd = match env::current_dir() {
        Ok(p) => p,
        Err(_) => return,
    };

    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let mut hostname_buf = [0i8; 256];
    let hostname = unsafe {
        if libc::gethostname(hostname_buf.as_mut_ptr(), hostname_buf.len()) == 0 {
            let len = hostname_buf.iter().position(|&c| c == 0).unwrap_or(hostname_buf.len());
            String::from_utf8_lossy(&hostname_buf[..len].iter().map(|&c| c as u8).collect::<Vec<_>>()).into_owned()
        } else {
            env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string())
        }
    };

    let pwd_str = pwd.display().to_string();
    let encoded: String = pwd_str
        .bytes()
        .flat_map(|b| -> Vec<u8> {
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.' || b == b'/' || b == b'~' {
                vec![b]
            } else {
                format!("%{:02X}", b).into_bytes()
            }
        })
        .map(|b| b as char)
        .collect();

    // Standard OSC 7 format with full hostname and ST (\x1b\\) terminator.
    let short_host = hostname.split('.').next().unwrap_or(&hostname);
    let seq7 = format!("\x1b]7;file://{}{}\x1b\\", hostname, encoded);

    let home = env::var("HOME").unwrap_or_default();
    let short_pwd = if !home.is_empty() && pwd_str.starts_with(&home) {
        if pwd_str == home {
            "~".to_string()
        } else {
            format!("~{}", &pwd_str[home.len()..])
        }
    } else {
        pwd_str.clone()
    };
    let seq0 = format!("\x1b]0;{}@{}:{}\x1b\\", user, short_host, short_pwd);

    let mut stdout = std::io::stdout();

    // If running inside a VTE terminal (Ptyxis / GNOME Console / GNOME Terminal),
    // emit VTE termprop signals (OSC 666) for native shell integration.
    let is_vte = env::var_os("VTE_VERSION").is_some();
    if is_vte {
        let _ = stdout.write_all(b"\x1b]666;vte.shell.postexec=0\x1b\\");
    }

    let _ = stdout.write_all(seq7.as_bytes());
    let _ = stdout.write_all(seq0.as_bytes());

    if is_vte {
        let _ = stdout.write_all(b"\x1b]666;vte.shell.precmd!\x1b\\");
    }

    let _ = stdout.flush();
}

static SHELL_TERMIOS: std::sync::Mutex<Option<libc::termios>> = std::sync::Mutex::new(None);

/// Saves the current shell termios settings (called at shell startup/interactive mode).
pub fn save_shell_termios() {
    unsafe {
        if libc::isatty(libc::STDIN_FILENO) != 0 {
            let mut termios: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(libc::STDIN_FILENO, &mut termios) == 0 {
                if let Ok(mut lock) = SHELL_TERMIOS.lock() {
                    *lock = Some(termios);
                }
            }
        }
    }
}

/// Restores the shell termios settings after a child process exits.
pub fn restore_shell_termios() {
    unsafe {
        if libc::isatty(libc::STDIN_FILENO) != 0 {
            if let Ok(lock) = SHELL_TERMIOS.lock() {
                if let Some(ref termios) = *lock {
                    libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, termios);
                }
            }
        }
    }
}

/// Resets terminal modes (mouse tracking, cursor visibility, bracketed paste)
/// and flushes unconsumed input from the kernel TTY input queue (`tcflush`).
pub fn reset_terminal_and_flush_stdin() {
    use std::io::IsTerminal;
    unsafe {
        if libc::isatty(libc::STDIN_FILENO) != 0 {
            if std::io::stdout().is_terminal() {
                use std::io::Write;
                let reset_seq = "\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?2004l\x1b[?25h";
                let mut stdout = std::io::stdout();
                let _ = stdout.write_all(reset_seq.as_bytes());
                let _ = stdout.flush();
            }
            libc::tcflush(libc::STDIN_FILENO, libc::TCIFLUSH);
        }
    }
}

/// Expands bash-style brace expressions like `{a,b}/{c,d}` into multiple strings.
/// Words without braces or without commas inside braces are returned unchanged.
pub fn expand_braces(s: &str) -> Vec<String> {
    if !s.contains('{') || !s.contains('}') || !s.contains(',') {
        return vec![s.to_string()];
    }

    let mut depth = 0;
    let mut start = None;
    let mut found_start = 0;
    let mut found_end = 0;
    let mut contains_comma = false;

    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();

    let mut i = 0;
    while i < n {
        let c = chars[i];
        if c == '\\' {
            i += 2;
            continue;
        }
        if c == '{' {
            if depth == 0 {
                start = Some(i);
                contains_comma = false;
            }
            depth += 1;
        } else if c == '}' && depth > 0 {
            depth -= 1;
            if depth == 0 {
                if contains_comma {
                    found_start = start.unwrap();
                    found_end = i;
                    break;
                } else {
                    start = None;
                }
            }
        } else if c == ',' && depth == 1 {
            contains_comma = true;
        }
        i += 1;
    }

    if start.is_none() || found_end == 0 {
        return vec![s.to_string()];
    }

    let prefix: String = chars[..found_start].iter().collect();
    let suffix: String = chars[found_end + 1..].iter().collect();
    let inner: String = chars[found_start + 1..found_end].iter().collect();

    let mut options = Vec::new();
    let mut current_opt = String::new();
    let mut inner_depth = 0;
    let inner_chars: Vec<char> = inner.chars().collect();

    let mut j = 0;
    while j < inner_chars.len() {
        let ic = inner_chars[j];
        if ic == '\\' {
            if j + 1 < inner_chars.len() {
                current_opt.push(ic);
                current_opt.push(inner_chars[j + 1]);
                j += 2;
                continue;
            }
        }
        if ic == '{' {
            inner_depth += 1;
            current_opt.push(ic);
        } else if ic == '}' {
            if inner_depth > 0 {
                inner_depth -= 1;
            }
            current_opt.push(ic);
        } else if ic == ',' && inner_depth == 0 {
            options.push(current_opt);
            current_opt = String::new();
        } else {
            current_opt.push(ic);
        }
        j += 1;
    }
    options.push(current_opt);

    let mut results = Vec::new();
    for opt in options {
        let candidate = format!("{}{}{}", prefix, opt, suffix);
        let expanded = expand_braces(&candidate);
        results.extend(expanded);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_braces() {
        assert_eq!(
            expand_braces("{deb-variant,fedora-variant}/{server,desktop}"),
            vec![
                "deb-variant/server",
                "deb-variant/desktop",
                "fedora-variant/server",
                "fedora-variant/desktop"
            ]
        );
        assert_eq!(expand_braces("{a,b,c}"), vec!["a", "b", "c"]);
        assert_eq!(expand_braces("pre_{1,2}_post"), vec!["pre_1_post", "pre_2_post"]);
        assert_eq!(expand_braces("plain"), vec!["plain"]);
        assert_eq!(expand_braces("{single}"), vec!["{single}"]);
    }
}
