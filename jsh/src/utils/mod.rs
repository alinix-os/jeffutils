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

/// Expands a leading `~` (or `~/`) to the value of `$HOME`.
pub fn expand_tilde(path: &str) -> String {
    let home = env::var("HOME").unwrap_or_default();
    expand_tilde_with(path, &home)
}

/// Expands environment variables and tilde in a path (used for redirect targets).
pub fn expand_target(path: &str) -> String {
    expand_tilde(&expand_env_vars(path))
}
