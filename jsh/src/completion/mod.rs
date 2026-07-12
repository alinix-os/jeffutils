use rustyline::CompletionType;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::hint::{Hinter, HistoryHinter, Hint};
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

use crate::builtin::{is_builtin, is_executable};
use crate::utils::expand_tilde;

/// Subcommands offered for well-known tools when completing their first
/// argument. Kept as a small hand-maintained table — enough to cover the
/// day-to-day verbs without shelling out to the tool.
fn known_subcommands(cmd: &str) -> Option<&'static [&'static str]> {
    Some(match cmd {
        "git" => &[
            "add", "branch", "checkout", "clone", "commit", "diff", "fetch",
            "init", "log", "merge", "pull", "push", "rebase", "remote",
            "reset", "restore", "stash", "status", "switch", "tag",
        ],
        "cargo" => &[
            "add", "bench", "build", "check", "clean", "clippy", "doc",
            "fmt", "init", "install", "new", "publish", "remove", "run",
            "test", "update",
        ],
        _ => return None,
    })
}

pub struct JshHint {
    display: String,
    complete: String,
}

impl Hint for JshHint {
    fn display(&self) -> &str {
        &self.display
    }
    fn completion(&self) -> Option<&str> {
        Some(&self.complete)
    }
}

pub struct JshHelper {
    pub hinter: HistoryHinter,
    pub completer: FilenameCompleter,
    pub aliases: Arc<Mutex<HashMap<String, String>>>,
}

impl Helper for JshHelper {}

impl Completer for JshHelper {
    type Candidate = Pair;
    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let prefix = &line[..pos];

        // The word currently under the cursor starts right after the last
        // unescaped whitespace. Everything before it is the "leading" part
        // of the command line, which tells us the argument position.
        let word_start = prefix
            .rfind(char::is_whitespace)
            .map(|i| i + 1)
            .unwrap_or(0);
        let word = &prefix[word_start..];
        let leading: Vec<&str> = prefix[..word_start].split_whitespace().collect();
        let arg_index = leading.len(); // 0 = completing the command itself
        let first_word = leading.first().copied().unwrap_or("");

        // `$VAR` completion works in any position, not just the command slot.
        if let Some(var_prefix) = word.strip_prefix('$') {
            let mut candidates = Vec::new();
            for (key, _) in env::vars() {
                if key.to_lowercase().starts_with(&var_prefix.to_lowercase()) {
                    candidates.push(Pair {
                        display: format!("${}", key),
                        replacement: format!("${}", key),
                    });
                }
            }
            candidates.sort_by(|a, b| a.display.cmp(&b.display));
            return Ok((word_start, candidates));
        }

        // Completing the command name itself (first word, no path separators).
        if arg_index == 0 && !word.contains('/') && !word.contains('\\') {
            let mut candidates = Vec::new();
            let wl = word.to_lowercase();

            let builtins = [
                "cd", "exit", "jeofetch", "help", "version", "export", "unset", "set",
                "alias", "unalias", "source", "true", "false", ".-1", "$PWD_BACK", "$PB",
            ];
            for b in builtins {
                if b.to_lowercase().starts_with(&wl) {
                    candidates.push(Pair { display: b.to_string(), replacement: b.to_string() });
                }
            }

            if let Ok(aliases) = self.aliases.lock() {
                for name in aliases.keys() {
                    if name.to_lowercase().starts_with(&wl) {
                        candidates.push(Pair { display: name.clone(), replacement: name.clone() });
                    }
                }
            }

            let path_var = env::var_os("PATH").unwrap_or_default();
            for path in env::split_paths(&path_var) {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        if name.to_lowercase().starts_with(&wl) && entry.path().is_file() {
                            candidates.push(Pair { display: name.clone(), replacement: name });
                        }
                    }
                }
            }

            candidates.sort_by(|a, b| a.display.cmp(&b.display));
            candidates.dedup_by(|a, b| a.display == b.display);
            if !candidates.is_empty() {
                return Ok((word_start, candidates));
            }
        }

        // Subcommand completion for known tools (git/cargo …) on their 1st arg.
        if arg_index == 1 {
            if let Some(subs) = known_subcommands(first_word) {
                let wl = word.to_lowercase();
                let mut candidates: Vec<Pair> = subs
                    .iter()
                    .filter(|s| s.to_lowercase().starts_with(&wl))
                    .map(|s| Pair { display: s.to_string(), replacement: s.to_string() })
                    .collect();
                if !candidates.is_empty() {
                    candidates.sort_by(|a, b| a.display.cmp(&b.display));
                    return Ok((word_start, candidates));
                }
            }
        }

        // `cd`/`pushd` take directories only — offer just those, with tilde
        // expansion so `cd ~/Desk<tab>` works.
        let dir_only = matches!(first_word, "cd" | "pushd") && arg_index == 1;
        if dir_only || word.starts_with('~') {
            if let Some(result) = self.complete_path(word, word_start, dir_only) {
                return Ok(result);
            }
        }

        // Fallback to rustyline's filename completion.
        self.completer.complete(line, pos, ctx)
    }
}

impl JshHelper {
    /// Completes a filesystem path, expanding a leading `~` for lookup while
    /// keeping it in the replacement text. When `dirs_only` is set, only
    /// directories are offered (used for `cd`). Returns `None` if the parent
    /// directory can't be read, so the caller can fall back.
    fn complete_path(&self, word: &str, word_start: usize, dirs_only: bool) -> Option<(usize, Vec<Pair>)> {
        let expanded = expand_tilde(word);

        // Split into "directory part" (already typed) and the fragment being
        // completed after the last '/'.
        let (dir_part, frag) = match expanded.rfind('/') {
            Some(i) => (&expanded[..=i], &expanded[i + 1..]),
            None => ("", expanded.as_str()),
        };
        // The visible prefix (with ~ intact) up to and including the last '/'.
        let visible_dir = match word.rfind('/') {
            Some(i) => &word[..=i],
            None => "",
        };

        let lookup_dir = if dir_part.is_empty() { "." } else { dir_part };
        let entries = fs::read_dir(lookup_dir).ok()?;

        let fl = frag.to_lowercase();
        let mut candidates = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.to_lowercase().starts_with(&fl) {
                continue;
            }
            let is_dir = entry.path().is_dir();
            if dirs_only && !is_dir {
                continue;
            }
            // Append '/' to directories so the next tab descends into them.
            let suffix = if is_dir { "/" } else { "" };
            let replacement = format!("{}{}{}", visible_dir, name, suffix);
            candidates.push(Pair { display: format!("{}{}", name, suffix), replacement });
        }
        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        Some((word_start, candidates))
    }
}

impl Hinter for JshHelper {
    type Hint = JshHint;
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        if let Some(h) = self.hinter.hint(line, pos, ctx) {
            let raw_text = h.completion()?.to_string();
            let display_text = format!("\x1B[90m{}\x1B[0m", raw_text);
            Some(JshHint {
                display: display_text,
                complete: raw_text,
            })
        } else {
            None
        }
    }
}

impl Validator for JshHelper {
    fn validate(&self, _ctx: &mut rustyline::validate::ValidationContext<'_>) -> rustyline::Result<rustyline::validate::ValidationResult> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

impl Highlighter for JshHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.is_empty() {
            return Cow::Borrowed(line);
        }
        
        let mut result = String::new();
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() {
            return Cow::Borrowed(line);
        }
        
        let mut current_idx = 0;
        let aliases_map = self.aliases.lock().unwrap();
        let first_word = words[0];

        for (i, word) in words.iter().enumerate() {
            if let Some(pos) = line[current_idx..].find(word) {
                result.push_str(&line[current_idx..current_idx + pos]);
                current_idx += pos + word.len();
            }
            
            let clean_word = if word.starts_with("~/") {
                word.replacen("~/", "", 1)
            } else if *word == "~" {
                "".to_string()
            } else {
                word.to_string()
            };

            if i == 0 {
                if *word == "texit" || *word == "nano" || is_executable(word) {
                    result.push_str(&format!("\x1B[32m{}\x1B[0m", word)); // Green
                } else if aliases_map.contains_key(*word) {
                    result.push_str(&format!("\x1B[38;5;208m{}\x1B[0m", word)); // Orange
                } else if is_builtin(word) {
                    result.push_str(&format!("\x1B[32m{}\x1B[0m", word)); // Green
                } else if Path::new(&clean_word).is_dir() || *word == "~" || *word == ".-1" || *word == "$PWD_BACK" || *word == "$PB" {
                    result.push_str(&format!("\x1B[34m{}\x1B[0m", word)); // Blue
                } else {
                    result.push_str(&format!("\x1B[31m{}\x1B[0m", word)); // Red
                }
            } else {
                if first_word == "nano" || first_word == "texit" {
                    result.push_str(&format!("\x1B[36m{}\x1B[0m", word)); // Cyan for editor arguments
                } else if Path::new(&clean_word).is_dir() || *word == "~" || *word == ".-1" || *word == "$PWD_BACK" || *word == "$PB" {
                    result.push_str(&format!("\x1B[34m{}\x1B[0m", word));
                } else {
                    result.push_str(word);
                }
            }
        }
        
        if current_idx < line.len() {
            result.push_str(&line[current_idx..]);
        }
        Cow::Owned(result)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }

    // Highlight the selected candidate in white background and black text (reverse video)
    fn highlight_candidate<'c>(&self, candidate: &'c str, _completion: CompletionType) -> Cow<'c, str> {
        Cow::Owned(format!("\x1B[7m{}\x1B[0m", candidate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn helper() -> JshHelper {
        JshHelper {
            hinter: HistoryHinter::new(),
            completer: FilenameCompleter::new(),
            aliases: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[test]
    fn subcommands_known() {
        assert!(known_subcommands("git").unwrap().contains(&"commit"));
        assert!(known_subcommands("cargo").unwrap().contains(&"build"));
        assert!(known_subcommands("nonesuch").is_none());
    }

    #[test]
    fn cd_offers_only_dirs() {
        // Build an isolated dir with one subdir and one file, then complete
        // its path with dirs_only=true and check only the dir shows up.
        let base = std::env::temp_dir().join(format!("jsh_ct_{}", std::process::id()));
        let _ = fs::create_dir_all(base.join("subdir"));
        let _ = fs::write(base.join("file.txt"), b"x");

        let h = helper();
        let word = format!("{}/", base.display());
        let (_, cands) = h.complete_path(&word, 0, true).unwrap();
        assert!(cands.iter().any(|p| p.display == "subdir/"),
            "expected subdir/ among {:?}", cands.iter().map(|p| &p.display).collect::<Vec<_>>());
        assert!(!cands.iter().any(|p| p.display.starts_with("file.txt")),
            "file.txt must not appear when dirs_only");

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn tilde_kept_in_replacement() {
        let h = helper();
        // "~/" should read $HOME and keep the ~/ prefix in every replacement.
        if let Some((_, cands)) = h.complete_path("~/", 0, false) {
            assert!(cands.iter().all(|p| p.replacement.starts_with("~/")));
        }
    }
}
