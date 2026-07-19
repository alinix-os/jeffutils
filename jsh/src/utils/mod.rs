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

    let mut candidates = vec![
        "cd".to_string(), "export".to_string(), "unset".to_string(),
        "set".to_string(), "alias".to_string(), "unalias".to_string(),
        "source".to_string(), "true".to_string(), "false".to_string(),
        "exec".to_string(), "exit".to_string(),
    ];

    if let Ok(aliases) = state.aliases.lock() {
        for alias in aliases.keys() {
            candidates.push(alias.clone());
        }
    }
    if let Ok(funcs) = state.functions.lock() {
        for func in funcs.keys() {
            candidates.push(func.clone());
        }
    }

    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for path in std::env::split_paths(&path_var) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() || file_type.is_symlink() {
                        if let Ok(name) = entry.file_name().into_string() {
                            candidates.push(name);
                        }
                    }
                }
            }
        }
    }

    let mut best_match = None;
    let mut min_distance = 3; // threshold
    
    for cand in candidates {
        let dist = strsim::damerau_levenshtein(cmd, &cand);
        if dist < min_distance {
            min_distance = dist;
            best_match = Some(cand);
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
