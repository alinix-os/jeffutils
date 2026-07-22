use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write, Seek};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crossterm::ExecutableCommand;
use crossterm::cursor::{MoveUp};
use crossterm::terminal::{Clear, ClearType};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: String, // ISO 8601
    pub cwd: String,
    pub exit_code: i32,
    pub count: usize,
    pub last_used: String, // ISO 8601
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    #[serde(default = "default_true")]
    pub history: bool,
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    #[serde(default = "default_true")]
    pub autosuggestion: bool,
    #[serde(default = "default_true")]
    pub fuzzy_history: bool,
    #[serde(default = "default_true")]
    pub share_history: bool,
    #[serde(default = "default_true")]
    pub history_sync: bool,
    #[serde(default = "default_true")]
    pub ignore_duplicates: bool,
}

fn default_true() -> bool { true }
fn default_history_size() -> usize { 100000 }

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            history: true,
            history_size: 100000,
            autosuggestion: true,
            fuzzy_history: true,
            share_history: true,
            history_sync: true,
            ignore_duplicates: true,
        }
    }
}

pub struct HistoryState {
    pub config: HistoryConfig,
    pub history_file_path: PathBuf,
    pub pins_file_path: PathBuf,
    pub entries: Vec<HistoryEntry>,
    pub cmd_to_idx: HashMap<String, usize>,
    pub pins: HashSet<String>,
    pub last_file_size: u64,
    pub last_mtime: Option<SystemTime>,
}

impl HistoryState {
    pub fn add_or_update_entry(&mut self, cmd: &str, timestamp: &str, cwd: &str, exit_code: i32) {
        if let Some(&idx) = self.cmd_to_idx.get(cmd) {
            let entry = &mut self.entries[idx];
            entry.count += 1;
            entry.last_used = timestamp.to_string();
            entry.exit_code = exit_code;
            entry.cwd = cwd.to_string();
        } else {
            let entry = HistoryEntry {
                command: cmd.to_string(),
                timestamp: timestamp.to_string(),
                cwd: cwd.to_string(),
                exit_code,
                count: 1,
                last_used: timestamp.to_string(),
            };
            self.entries.push(entry);
            self.cmd_to_idx.insert(cmd.to_string(), self.entries.len() - 1);
        }
    }

    pub fn save_all_history(&self) -> io::Result<()> {
        if let Some(parent) = self.history_file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut file = File::create(&self.history_file_path)?;
        for entry in &self.entries {
            let json = serde_json::to_string(entry)?;
            writeln!(file, "{}", json)?;
        }
        Ok(())
    }

    pub fn load_pins(&mut self) -> io::Result<()> {
        if !self.pins_file_path.exists() {
            return Ok(());
        }
        let file = File::open(&self.pins_file_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                self.pins.insert(trimmed.to_string());
            }
        }
        Ok(())
    }

    pub fn save_pins(&self) -> io::Result<()> {
        if let Some(parent) = self.pins_file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut file = File::create(&self.pins_file_path)?;
        for cmd in &self.pins {
            writeln!(file, "{}", cmd)?;
        }
        Ok(())
    }

    pub fn sync_history(&mut self) -> io::Result<()> {
        if !self.config.history_sync {
            return Ok(());
        }
        let metadata = match fs::metadata(&self.history_file_path) {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };
        let mtime = metadata.modified().ok();
        let size = metadata.len();

        if self.last_mtime == mtime && self.last_file_size == size {
            return Ok(());
        }

        if size < self.last_file_size {
            self.entries.clear();
            self.cmd_to_idx.clear();
            self.load_from_file_internal()?;
            return Ok(());
        }

        let mut file = File::open(&self.history_file_path)?;
        file.seek(io::SeekFrom::Start(self.last_file_size))?;

        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(trimmed) {
                self.add_or_update_entry(
                    &entry.command,
                    &entry.timestamp,
                    &entry.cwd,
                    entry.exit_code,
                );
            }
        }

        self.last_file_size = size;
        self.last_mtime = mtime;
        Ok(())
    }

    pub fn load_from_file_internal(&mut self) -> io::Result<()> {
        if !self.history_file_path.exists() {
            return Ok(());
        }
        let file = File::open(&self.history_file_path)?;
        let metadata = file.metadata()?;
        self.last_file_size = metadata.len();
        self.last_mtime = metadata.modified().ok();

        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(trimmed) {
                self.add_or_update_entry(
                    &entry.command,
                    &entry.timestamp,
                    &entry.cwd,
                    entry.exit_code,
                );
            }
        }
        Ok(())
    }
}

pub struct HistoryManager {
    pub state: Arc<Mutex<HistoryState>>,
}

impl HistoryManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").ok().map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let history_file_path = home.join(".local/share/jsh/history");
        let pins_file_path = home.join(".local/share/jsh/pins");

        let state = Arc::new(Mutex::new(HistoryState {
            config: HistoryConfig::default(),
            history_file_path,
            pins_file_path,
            entries: Vec::new(),
            cmd_to_idx: HashMap::new(),
            pins: HashSet::new(),
            last_file_size: 0,
            last_mtime: None,
        }));

        Self { state }
    }

    pub fn load_history(&self) {
        let mut state = self.state.lock().unwrap();
        state.config = load_config();

        let file_exists = state.history_file_path.exists();
        let _ = state.load_from_file_internal();

        if !file_exists || state.entries.is_empty() {
            import_legacy_history(&mut state);
            let _ = state.save_all_history();
        }

        let _ = state.load_pins();
    }

    pub fn add_entry(&self, command: &str, exit_code: i32, cwd: &str) {
        let mut state = self.state.lock().unwrap();
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return;
        }

        if matches!(trimmed, "history" | "clear" | "exit") {
            return;
        }

        if let Ok(histignore) = std::env::var("HISTIGNORE") {
            let mut matches_ignore = false;
            for pattern in histignore.split(':') {
                if glob_match(trimmed, pattern) {
                    matches_ignore = true;
                    break;
                }
            }
            if matches_ignore {
                return;
            }
        }

        let now = Local::now().to_rfc3339();

        if state.config.ignore_duplicates {
            if let Some(last_entry) = state.entries.last() {
                if last_entry.command == trimmed {
                    let idx = state.entries.len() - 1;
                    let entry = &mut state.entries[idx];
                    entry.last_used = now.clone();
                    entry.exit_code = exit_code;
                    entry.cwd = cwd.to_string();
                    entry.count += 1;

                    let entry_to_write = entry.clone();
                    let file_path = state.history_file_path.clone();
                    std::thread::spawn(move || {
                        let _ = append_entry_to_file(&file_path, &entry_to_write);
                    });
                    return;
                }
            }
        }

        state.add_or_update_entry(trimmed, &now, cwd, exit_code);

        if let Some(&idx) = state.cmd_to_idx.get(trimmed) {
            let entry_to_write = state.entries[idx].clone();
            let file_path = state.history_file_path.clone();
            std::thread::spawn(move || {
                let _ = append_entry_to_file(&file_path, &entry_to_write);
            });
        }
    }

    pub fn get_suggestion(&self, query: &str, current_dir: &str) -> Option<String> {
        let mut state = self.state.lock().unwrap();
        let _ = state.sync_history();

        let total = state.entries.len();
        if total == 0 || query.is_empty() {
            return None;
        }

        let mut best_score = -1.0;
        let mut best_cmd = None;

        for (idx, entry) in state.entries.iter().enumerate() {
            if entry.command.starts_with(query) && entry.command != query {
                let is_pinned = state.pins.contains(&entry.command);
                let dir_score = if entry.cwd == current_dir { 1.0 } else { 0.0 };
                let recency_score = (idx + 1) as f64 / total as f64 * 100.0;
                let freq_score = entry.count as f64;

                let mut score = 100.0
                    + (dir_score * 50.0)
                    + (freq_score * 0.6)
                    + (recency_score * 0.4);

                if is_pinned {
                    score += 10000.0;
                }

                if score > best_score {
                    best_score = score;
                    best_cmd = Some(entry.command.clone());
                }
            }
        }

        best_cmd
    }

    pub fn get_navigation_entries(&self, query: &str, current_dir: &str) -> Vec<String> {
        let mut state = self.state.lock().unwrap();
        let _ = state.sync_history();

        let mut local = Vec::new();
        let mut global = Vec::new();
        let mut seen = HashSet::new();

        for entry in state.entries.iter().rev() {
            if entry.command.starts_with(query) {
                if !seen.insert(entry.command.clone()) {
                    continue;
                }
                if entry.cwd == current_dir {
                    local.push(entry.command.clone());
                } else {
                    global.push(entry.command.clone());
                }
            }
        }

        let mut combined = local;
        combined.extend(global);
        combined
    }

    pub fn search(&self, query: &str, cwd: &str, limit: usize) -> Vec<String> {
        let state = self.state.lock().unwrap();
        let mut matches = Vec::new();
        let total = state.entries.len();
        if total == 0 {
            return Vec::new();
        }

        for (idx, entry) in state.entries.iter().enumerate() {
            let is_pinned = state.pins.contains(&entry.command);
            let score = calculate_reverse_search_score(
                &entry.command,
                query,
                cwd,
                &entry.cwd,
                is_pinned,
                idx,
                total,
                entry.count,
            );
            if score >= 0.0 {
                matches.push((entry.command.clone(), score));
            }
        }

        matches.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        let mut seen = HashSet::new();
        let mut deduped = Vec::new();
        for (cmd, _) in matches {
            if seen.insert(cmd.clone()) {
                deduped.push(cmd);
                if deduped.len() >= limit {
                    break;
                }
            }
        }
        deduped
    }

    pub fn pin_command(&self, cmd: &str) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.pins.insert(cmd.to_string());
        state.save_pins()
    }

    pub fn unpin_command(&self, cmd: &str) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.pins.remove(cmd);
        state.save_pins()
    }

    pub fn clear_history(&self) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.entries.clear();
        state.cmd_to_idx.clear();
        if state.history_file_path.exists() {
            let _ = fs::remove_file(&state.history_file_path);
        }
        state.last_file_size = 0;
        state.last_mtime = None;
        Ok(())
    }

    pub fn print_history(&self) {
        let state = self.state.lock().unwrap();
        for (i, entry) in state.entries.iter().enumerate() {
            let id = i + 1;
            let status_colored = if entry.exit_code == 0 {
                format!("\x1B[32m{}\x1B[0m", entry.exit_code)
            } else {
                format!("\x1B[31m{}\x1B[0m", entry.exit_code)
            };
            println!(
                "{:>5} | {} | {} | {} | {}",
                id,
                entry.timestamp,
                status_colored,
                entry.cwd,
                entry.command
            );
        }
    }
}

fn glob_match(cmd: &str, pattern: &str) -> bool {
    fn match_helper(cmd_chars: &[char], pat_chars: &[char]) -> bool {
        match (cmd_chars, pat_chars) {
            ([], []) => true,
            (_, ['*']) => true,
            ([], _) => false,
            ([c, cmd_tail @ ..], [p, pat_tail @ ..]) => {
                if p == &'*' {
                    match_helper(cmd_chars, pat_tail) || match_helper(cmd_tail, pat_chars)
                } else if p == &'?' || p == c {
                    match_helper(cmd_tail, pat_tail)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    let cmd_chars: Vec<char> = cmd.chars().collect();
    let pat_chars: Vec<char> = pattern.chars().collect();
    match_helper(&cmd_chars, &pat_chars)
}

fn is_fuzzy_match(command: &str, query: &str) -> bool {
    let mut cmd_chars = command.chars();
    for q_char in query.chars() {
        if q_char.is_whitespace() {
            continue;
        }
        let q_lower = q_char.to_lowercase().next().unwrap();
        loop {
            match cmd_chars.next() {
                Some(c) => {
                    if c.to_lowercase().next().unwrap() == q_lower {
                        break;
                    }
                }
                None => return false,
            }
        }
    }
    true
}

fn calculate_reverse_search_score(
    command: &str,
    query: &str,
    cwd: &str,
    entry_cwd: &str,
    is_pinned: bool,
    entry_idx: usize,
    total_entries: usize,
    count: usize,
) -> f64 {
    let query_lower = query.to_lowercase();
    let command_lower = command.to_lowercase();

    let match_score = if query_lower.is_empty() {
        1.0
    } else if command_lower.starts_with(&query_lower) {
        100.0
    } else if command_lower.contains(&query_lower) {
        50.0
    } else if is_fuzzy_match(&command_lower, &query_lower) {
        10.0
    } else {
        return -1.0;
    };

    let dir_score = if entry_cwd == cwd { 50.0 } else { 0.0 };
    let recency_score = (entry_idx + 1) as f64 / total_entries as f64 * 40.0;
    let freq_score = count as f64 * 0.6;

    let mut score = match_score + dir_score + recency_score + freq_score;
    if is_pinned {
        score += 10000.0;
    }
    score
}

fn append_entry_to_file(path: &Path, entry: &HistoryEntry) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let json = serde_json::to_string(entry)?;
    writeln!(file, "{}", json)?;
    Ok(())
}

fn load_config() -> HistoryConfig {
    let home = std::env::var("HOME").ok().map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let config_path = home.join(".config/jsh/config.toml");
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            #[derive(Deserialize)]
            struct RootConfig {
                history: Option<HistoryConfig>,
            }
            if let Ok(parsed) = toml::from_str::<RootConfig>(&content) {
                if let Some(cfg) = parsed.history {
                    return cfg;
                }
            } else if let Ok(parsed) = toml::from_str::<HistoryConfig>(&content) {
                return parsed;
            }
        }
    } else {
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let default_config = r#"[history]
history = true
history_size = 100000
autosuggestion = true
fuzzy_history = true
share_history = true
history_sync = true
ignore_duplicates = true
"#;
        let _ = fs::write(&config_path, default_config);
    }
    HistoryConfig::default()
}

fn import_legacy_history(state: &mut HistoryState) {
    let home = std::env::var("HOME").ok().map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let bash_path = home.join(".bash_history");
    if bash_path.exists() {
        if let Ok(file) = File::open(&bash_path) {
            let reader = BufReader::new(file);
            let mut lines = reader.lines().flatten().peekable();
            while let Some(line) = lines.next() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let mut timestamp = Local::now().to_rfc3339();
                let mut cmd = trimmed.to_string();

                if trimmed.starts_with('#') {
                    if let Ok(ts_secs) = trimmed[1..].parse::<i64>() {
                        if let Some(dt) = DateTime::from_timestamp(ts_secs, 0) {
                            let local_dt: DateTime<Local> = dt.into();
                            timestamp = local_dt.to_rfc3339();
                        }
                        if let Some(next_line) = lines.next() {
                            cmd = next_line.trim().to_string();
                        } else {
                            break;
                        }
                    }
                }

                state.add_or_update_entry(&cmd, &timestamp, &home.to_string_lossy(), 0);
            }
        }
    }

    let zsh_path = home.join(".zsh_history");
    if zsh_path.exists() {
        if let Ok(file) = File::open(&zsh_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if line.starts_with(':') {
                    let parts: Vec<&str> = line.splitn(3, ':').collect();
                    if parts.len() == 3 {
                        let ts_str = parts[1].trim();
                        let rest = parts[2];
                        let mut cmd = rest.to_string();
                        let mut timestamp = Local::now().to_rfc3339();

                        if let Ok(ts_secs) = ts_str.parse::<i64>() {
                            if let Some(dt) = DateTime::from_timestamp(ts_secs, 0) {
                                let local_dt: DateTime<Local> = dt.into();
                                timestamp = local_dt.to_rfc3339();
                            }
                        }

                        if let Some(semi_idx) = rest.find(';') {
                            cmd = rest[semi_idx + 1..].to_string();
                        }

                        let cmd_trimmed = cmd.trim();
                        if !cmd_trimmed.is_empty() {
                            state.add_or_update_entry(cmd_trimmed, &timestamp, &home.to_string_lossy(), 0);
                        }
                    }
                } else {
                    let cmd_trimmed = line.trim();
                    if !cmd_trimmed.is_empty() {
                        let timestamp = Local::now().to_rfc3339();
                        state.add_or_update_entry(cmd_trimmed, &timestamp, &home.to_string_lossy(), 0);
                    }
                }
            }
        }
    }
}

pub fn interactive_reverse_search(
    history_mgr: &HistoryManager,
    cwd: &str,
) -> io::Result<Option<String>> {
    let mut query = String::new();
    let mut selected_index = 0;

    println!();

    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;

    let mut last_num_lines = 0;

    loop {
        let limit = 5;
        let matches = history_mgr.search(&query, cwd, limit);

        if selected_index >= matches.len() {
            selected_index = matches.len().saturating_sub(1);
        }

        if last_num_lines > 0 {
            for _ in 0..last_num_lines {
                stdout.execute(MoveUp(1))?;
                stdout.execute(Clear(ClearType::CurrentLine))?;
            }
        }

        print!("\r\x1B[1;36m(reverse-i-search)\x1B[0m '{}':", query);
        stdout.execute(Clear(ClearType::UntilNewLine))?;
        println!();

        let mut num_lines = 1;
        for (i, m) in matches.iter().enumerate() {
            if i == selected_index {
                print!("\r  \x1B[1;32m>\x1B[0m {} ", m);
            } else {
                print!("\r    {} ", m);
            }
            stdout.execute(Clear(ClearType::UntilNewLine))?;
            println!();
            num_lines += 1;
        }

        stdout.flush()?;
        last_num_lines = num_lines;

        if event::poll(std::time::Duration::from_millis(500))? {
            if let CEvent::Key(key_event) = event::read()? {
                if key_event.kind == event::KeyEventKind::Press {
                    match (key_event.code, key_event.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
                            if last_num_lines > 0 {
                                for _ in 0..last_num_lines {
                                    stdout.execute(MoveUp(1))?;
                                    stdout.execute(Clear(ClearType::CurrentLine))?;
                                }
                            }
                            stdout.execute(MoveUp(1))?;
                            crossterm::terminal::disable_raw_mode()?;
                            return Ok(None);
                        }
                        (KeyCode::Enter, _) => {
                            let selected_cmd = if matches.is_empty() {
                                None
                            } else {
                                Some(matches[selected_index].clone())
                            };

                            if last_num_lines > 0 {
                                for _ in 0..last_num_lines {
                                    stdout.execute(MoveUp(1))?;
                                    stdout.execute(Clear(ClearType::CurrentLine))?;
                                }
                            }
                            stdout.execute(MoveUp(1))?;
                            crossterm::terminal::disable_raw_mode()?;
                            return Ok(selected_cmd);
                        }
                        (KeyCode::Up, _) => {
                            if selected_index > 0 {
                                selected_index -= 1;
                            }
                        }
                        (KeyCode::Down, _) => {
                            if selected_index + 1 < matches.len() {
                                selected_index += 1;
                            }
                        }
                        (KeyCode::Backspace, _) => {
                            query.pop();
                            selected_index = 0;
                        }
                        (KeyCode::Char(c), m) if m == KeyModifiers::NONE || m == KeyModifiers::SHIFT => {
                            query.push(c);
                            selected_index = 0;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matching() {
        assert!(glob_match("git status", "git*"));
        assert!(glob_match("git status", "*status"));
        assert!(glob_match("ls -la", "ls?*"));
        assert!(!glob_match("ls -la", "git*"));
    }

    #[test]
    fn test_fuzzy_matching() {
        assert!(is_fuzzy_match("git push", "gp"));
        assert!(is_fuzzy_match("git status", "g s"));
        assert!(!is_fuzzy_match("git status", "gps"));
    }

    #[test]
    fn test_history_addition_and_ranking() {
        let temp_dir = std::env::temp_dir().join(format!("jsh_hist_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let hist_file = temp_dir.join("history");
        let pins_file = temp_dir.join("pins");

        let state = Arc::new(Mutex::new(HistoryState {
            config: HistoryConfig {
                history: true,
                history_size: 100,
                autosuggestion: true,
                fuzzy_history: true,
                share_history: true,
                history_sync: true,
                ignore_duplicates: true,
            },
            history_file_path: hist_file.clone(),
            pins_file_path: pins_file.clone(),
            entries: Vec::new(),
            cmd_to_idx: HashMap::new(),
            pins: HashSet::new(),
            last_file_size: 0,
            last_mtime: None,
        }));

        let mgr = HistoryManager { state };

        mgr.add_entry("git commit", 0, "/dir1");
        mgr.add_entry("git push", 0, "/dir2");
        mgr.add_entry("git status", 0, "/dir1");

        let sug = mgr.get_suggestion("git", "/dir1");
        assert!(sug.is_some());
        assert_eq!(sug.unwrap(), "git status");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
