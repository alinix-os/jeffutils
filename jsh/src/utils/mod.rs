//! Shared utility helpers for `jsh`.
//!
//! Cross-module helpers (string/path expansion) used by several subsystems.

use std::env;

/// Expands `$VAR` and `${VAR}` references using the process environment.
pub fn expand_env_vars(arg: &str) -> String {
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
                result.push_str(&env::var(&var_name).unwrap_or_default());
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Expands a leading `~` (or `~/`) to the value of `$HOME`.
pub fn expand_tilde(path: &str) -> String {
    if path == "~" {
        env::var("HOME").unwrap_or_else(|_| path.to_string())
    } else if let Some(rest) = path.strip_prefix("~/") {
        let home = env::var("HOME").unwrap_or_default();
        format!("{}/{}", home, rest)
    } else {
        path.to_string()
    }
}

/// Expands environment variables and tilde in a path (used for redirect targets).
pub fn expand_target(path: &str) -> String {
    expand_tilde(&expand_env_vars(path))
}
