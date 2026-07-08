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
        let words: Vec<&str> = prefix.split_whitespace().collect();
        let is_command = words.len() <= 1 && !prefix.contains('/') && !prefix.contains('\\');

        if is_command {
            let mut candidates = Vec::new();
            let word_to_complete = words.first().copied().unwrap_or(prefix).trim_start();

            // Complete environment variables if prefix starts with $
            if word_to_complete.starts_with('$') {
                let var_prefix = &word_to_complete[1..];
                for (key, _) in env::vars() {
                    if key.to_lowercase().starts_with(&var_prefix.to_lowercase()) {
                        candidates.push(Pair {
                            display: format!("${}", key),
                            replacement: format!("${}", key),
                        });
                    }
                }
                candidates.sort_by(|a, b| a.display.cmp(&b.display));
                let offset = prefix.len() - word_to_complete.len();
                return Ok((offset, candidates));
            }

            // Complete builtins
            let builtins = vec!["cd", "exit", "jeofetch", "help", "version", ".-1", "$PWD_BACK", "$PB"];
            for builtin in builtins {
                if builtin.to_lowercase().starts_with(&word_to_complete.to_lowercase()) {
                    candidates.push(Pair {
                        display: builtin.to_string(),
                        replacement: builtin.to_string(),
                    });
                }
            }

            // Complete aliases
            if let Ok(aliases) = self.aliases.lock() {
                for alias_name in aliases.keys() {
                    if alias_name.to_lowercase().starts_with(&word_to_complete.to_lowercase()) {
                        candidates.push(Pair {
                            display: alias_name.clone(),
                            replacement: alias_name.clone(),
                        });
                    }
                }
            }

            // Complete executables in PATH
            let path_var = env::var_os("PATH").unwrap_or_default();
            for path in env::split_paths(&path_var) {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        if name.to_lowercase().starts_with(&word_to_complete.to_lowercase()) {
                            if entry.path().is_file() {
                                candidates.push(Pair {
                                    display: name.clone(),
                                    replacement: name,
                                });
                            }
                        }
                    }
                }
            }

            // Deduplicate and sort
            candidates.sort_by(|a, b| a.display.cmp(&b.display));
            candidates.dedup_by(|a, b| a.display == b.display);

            if !candidates.is_empty() {
                let offset = prefix.len() - word_to_complete.len();
                return Ok((offset, candidates));
            }
        }

        // Fallback to filename completion
        self.completer.complete(line, pos, ctx)
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
                // If it is a real system executable, it must always show green
                if is_executable(word) {
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
                if Path::new(&clean_word).is_dir() || *word == "~" || *word == ".-1" || *word == "$PWD_BACK" || *word == "$PB" {
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
