use std::io::Write;
use std::os::linux::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

/// Result of evaluating how dangerous a removal target is.
#[derive(Debug, PartialEq)]
pub enum Risk {
    /// Ordinary target, normal single confirmation applies.
    Normal,
    /// Dangerous target that requires an extra, explicit confirmation
    /// (the user must type the full path) even when `--force` is given.
    Critical(String),
    /// Target that must never be removed by this tool under any flag.
    Forbidden(String),
}

/// Absolute paths that must never be removed.
const FORBIDDEN_PATHS: &[&str] = &[
    "/", "/bin", "/boot", "/dev", "/etc", "/lib", "/lib32", "/lib64",
    "/proc", "/root", "/run", "/sbin", "/sys", "/usr", "/var", "/home",
    "/opt", "/srv", "/mnt", "/media",
];

/// Expand a leading `~` to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    } else if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Resolve a path to an absolute, normalized form without requiring it to
/// exist. Symlinks are not followed for non-existent tails, but `..` and `.`
/// components are collapsed so that tricks like `foo/../../../..` are caught.
fn normalize(path: &Path) -> PathBuf {
    // If it exists, prefer the canonical (symlink-resolved) path.
    if let Ok(canon) = std::fs::canonicalize(path) {
        return canon;
    }

    let base = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
    };

    let mut result = base;
    for component in path.components() {
        match component {
            Component::Prefix(p) => result.push(p.as_os_str()),
            Component::RootDir => {
                result.push(Component::RootDir.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(part) => result.push(part),
        }
    }
    result
}

/// Detect shell-glob metacharacters. If the shell did not expand them (e.g.
/// quoted, or no matches with `nullglob` off) they arrive here literally and
/// almost always signal a mistake.
fn looks_like_glob(raw: &str) -> bool {
    raw.contains('*') || raw.contains('?') || raw.contains('[')
}

/// Classify the risk of removing `raw_path`.
pub fn assess(raw_path: &str) -> Risk {
    if looks_like_glob(raw_path) {
        return Risk::Forbidden(format!(
            "target '{}' contains wildcard characters (* ? [). Refusing to \
             operate on an unexpanded glob — remove items explicitly.",
            raw_path
        ));
    }

    let expanded = expand_tilde(raw_path);
    let normalized = normalize(&expanded);
    let display = normalized.to_string_lossy().to_string();

    // Never touch the filesystem root or top-level system directories.
    for forbidden in FORBIDDEN_PATHS {
        if normalized == Path::new(forbidden) {
            return Risk::Forbidden(format!(
                "'{}' is a protected system path and cannot be removed",
                display
            ));
        }
    }

    // The user's own home directory: allowed, but critical.
    if let Some(home) = home_dir() {
        if normalized == home {
            return Risk::Critical(display);
        }
    }

    // A shallow absolute path (e.g. /something directly under root) is
    // treated as critical: one wrong recursive delete there is catastrophic.
    if normalized.is_absolute() {
        let depth = normalized
            .components()
            .filter(|c| matches!(c, Component::Normal(_)))
            .count();
        if depth <= 1 {
            return Risk::Critical(display);
        }
    }

    #[cfg(unix)]
    if let Ok(meta) = std::fs::metadata(&normalized) {
        if meta.st_nlink() > 1 {
            return Risk::Critical(display);
        }
    }

    Risk::Normal
}

/// Prompt requiring the user to type the exact resolved path to proceed.
/// Returns true only on an exact match. Used for `Critical` targets.
pub fn confirm_exact(resolved_path: &str) -> bool {
    eprintln!(
        "\x1b[1;31mDANGER:\x1b[0m you are about to remove '{}'.",
        resolved_path
    );
    eprintln!("This action is irreversible and may destroy important data.");
    print!(
        "To confirm, type the full path exactly ('{}'): ",
        resolved_path
    );
    std::io::stdout().flush().ok();

    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer).is_err() {
        return false;
    }

    answer.trim() == resolved_path
}
