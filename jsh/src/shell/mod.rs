use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crossterm::style::Stylize;

use crate::parser::{Word, WordSegment};

pub mod history;

pub struct ShellState {
    pub last_exit_status: i32,
    pub home_dir: PathBuf,
    pub init_info: bool,
    pub aliases: Arc<Mutex<HashMap<String, String>>>,
    pub old_pwd: Option<PathBuf>,
    /// Shell-local variables (`NAME=value`), distinct from process env vars.
    /// Looked up before falling back to `env::var`.
    pub shell_vars: Arc<Mutex<HashMap<String, String>>>,
    /// Names of shell vars that have been `export`ed to the process env.
    pub exported: HashSet<String>,
    /// Name jsh was invoked as / script path, used for `$0`.
    pub arg0: String,
    /// When true, "command not found" errors are swallowed instead of
    /// printed. Used while loading `.jshrc`, since it may contain bash-only
    /// constructs (functions, `[ ]` tests) this shell doesn't parse — each
    /// such line fails as an unknown command, and printing all of those on
    /// every startup would be noisy for configs migrated from bash/zsh.
    pub quiet_errors: bool,
    /// Paths passed to `source`/`.` that look like real bash scripts
    /// (define functions, use `[[`, etc.) rather than simple jsh-style
    /// config. jsh can't interpret bash functions itself, so commands that
    /// turn out to be unknown are retried through `bash -ic "source <file>;
    /// <cmd> <args>"` for each of these files — this is how things like
    /// `nvm use 18` keep working after `.jshrc` sources nvm.sh.
    pub bash_sourced_files: Vec<PathBuf>,
    /// User-defined shell functions (`name() { body }`), keyed by name.
    /// The body is the raw text between `{` and `}`, run as a nested
    /// script with `$1`, `$2`, ... bound to the call's arguments.
    pub functions: Arc<Mutex<HashMap<String, String>>>,
    /// Stack of positional-parameter frames for nested function calls;
    /// the top frame is used to resolve `$1`, `$2`, `$@`, `$#` while a
    /// function body is executing.
    positional_stack: Vec<Vec<String>>,
    /// Last-seen modification time of `.jshrc`, used to detect edits for
    /// hot-reloading. `None` until the file is first loaded.
    jshrc_mtime: Option<SystemTime>,
    /// Cached OS logo (emoji) for the prompt, populated on first access.
    cached_os_logo: Option<String>,
    /// Cached commands known to NOT exist in bash (neg cache for try_bash_fallback).
    /// This avoids spawning bash for every unknown command.
    bash_cmd_neg_cache: HashSet<String>,
    /// Whether the shell is currently running in an interactive session.
    pub is_interactive: bool,
    pub history_mgr: Arc<history::HistoryManager>,
}

impl ShellState {
    pub fn new() -> Self {
        let home = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));

        // Ensure PATH is set to a reasonable default if not present
        // This handles cases where PATH is empty or missing (e.g. when invoked via `sh -c`)
        let default_path = "/usr/local/bin:/usr/bin:/bin";
        let path = env::var_os("PATH");
        if path.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
            unsafe {
                env::set_var("PATH", default_path);
            }
        }

        // Override SHELL env var to point to our jsh
        if let Ok(exe) = env::current_exe() {
            unsafe {
                env::set_var("SHELL", exe);
            }
        }

        let aliases_map = Arc::new(Mutex::new(HashMap::new()));

        {
            let mut map = aliases_map.lock().unwrap();
            map.insert("ls".to_string(), "ls --color=auto".to_string());
            map.insert("grep".to_string(), "grep --color=auto".to_string());
            map.insert("ll".to_string(), "ls -la --color=auto".to_string());
            map.insert("c".to_string(), "clear".to_string());
        }

        let history_mgr = Arc::new(history::HistoryManager::new());
        history_mgr.load_history();

        Self {
            last_exit_status: 0,
            home_dir: home,
            init_info: true,
            aliases: aliases_map,
            old_pwd: None,
            shell_vars: Arc::new(Mutex::new(HashMap::new())),
            exported: HashSet::new(),
            arg0: "jsh".to_string(),
            quiet_errors: false,
            bash_sourced_files: Vec::new(),
            functions: Arc::new(Mutex::new(HashMap::new())),
            positional_stack: Vec::new(),
            jshrc_mtime: None,
            cached_os_logo: None,
            bash_cmd_neg_cache: HashSet::new(),
            is_interactive: false,
            history_mgr,
        }
    }

    pub fn load_jshrc(&mut self) {
        let jshrc_path = self.home_dir.join(".jshrc");
        if !jshrc_path.exists() {
            let default_jshrc = "\
# jsh configuration file
INIT_INFO=true

# When true, editing this file re-loads it automatically before each
# prompt — no need to `source .jshrc` or restart the shell.
HOT_RELOAD=true

# When true, shows elapsed time for commands that take >= 2s.
# Set to false to disable the \"(⏳ demorou Xs)\" notification.
SHOW_TIMING=true

alias c=\"clear\"
alias ls=\"ls --color=auto\"
alias grep=\"grep --color=auto\"

# Custom Exports
export EDITOR=texit
# Extend the inherited PATH instead of replacing it:
export PATH=$PATH:/usr/local/bin
";
            let _ = fs::write(&jshrc_path, default_jshrc);
        }

        self.jshrc_mtime = fs::metadata(&jshrc_path)
            .and_then(|m| m.modified())
            .ok();

        if let Ok(content) = fs::read_to_string(&jshrc_path) {
            self.quiet_errors = true;
            self.run_script_text(&content);
            self.quiet_errors = false;
        }
    }

    /// If hot-reload is enabled (`HOT_RELOAD=true` in `.jshrc`) and the file
    /// has been modified since it was last loaded, re-run it so edits take
    /// effect without `source .jshrc` or restarting the shell. Called before
    /// each interactive prompt. The `HOT_RELOAD` flag is read from the
    /// *currently loaded* config, so setting it to false (or removing it)
    /// and reloading once disables further auto-reloading.
    pub fn maybe_hot_reload(&mut self) {
        if self.get_var("HOT_RELOAD") != "true" {
            return;
        }
        let jshrc_path = self.home_dir.join(".jshrc");
        let Some(mtime) = fs::metadata(&jshrc_path).and_then(|m| m.modified()).ok() else {
            return;
        };
        if self.jshrc_mtime == Some(mtime) {
            return;
        }
        self.load_jshrc();
    }

    /// Heuristic: does `content` use bash syntax jsh genuinely can't parse
    /// (`[[ ]]`, `local`, `case`, etc — simple one-line function defs are
    /// now natively supported, see `run_script_text`)? If so, `source`/`.`
    /// should remember the file so unknown commands can be retried through
    /// real bash.
    pub fn looks_like_bash(content: &str) -> bool {
        content.contains("[[")
            || content.contains("local ")
            || content.lines().any(|l| l.trim().starts_with("case "))
    }

    /// Retries `program args...` through `bash -ic`, sourcing every bash
    /// script previously loaded via `source`/`.`, so functions defined
    /// there (e.g. `nvm`) remain callable from jsh. Returns `None` if there
    /// are no bash-sourced files or bash isn't available.
    /// Uses a negative cache to avoid repeated spawns for commands known to not exist.
    pub fn try_bash_fallback(&mut self, program: &str, args: &[String]) -> Option<i32> {
        if self.bash_sourced_files.is_empty() {
            return None;
        }

        // Check negative cache first - skip bash spawn if we already know this command doesn't exist
        if self.bash_cmd_neg_cache.contains(program) {
            return None;
        }

        let mut script = String::new();
        for f in &self.bash_sourced_files {
            script.push_str("source ");
            script.push('\'');
            script.push_str(&f.to_string_lossy().replace('\'', "'\\''"));
            script.push_str("' >/dev/null 2>&1; ");
        }

        let mut check_script = script.clone();
        check_script.push_str("type -t ");
        check_script.push_str(program);
        check_script.push_str(" >/dev/null 2>&1");

        let check_status = Command::new("bash")
            .arg("-c")
            .arg(&check_script)
            .status()
            .ok()?;

        if !check_status.success() {
            // Cache failure to avoid repeated spawns for the same missing command
            self.bash_cmd_neg_cache.insert(program.to_string());
            return None;
        }

        script.push_str(program);
        for a in args {
            script.push(' ');
            script.push('\'');
            script.push_str(&a.replace('\'', "'\\''"));
            script.push('\'');
        }

        let status = Command::new("bash")
            .arg("-ic")
            .arg(&script)
            .status()
            .ok()?;
        Some(status.code().unwrap_or(1))
    }

    /// Runs a block of script text line by line through the same
    /// tokenize -> parse -> expand -> execute pipeline used interactively,
    /// without requiring a TTY. Used for `.jshrc`, `source`, and non-interactive
    /// stdin/script invocation. Also recognizes and stores simple shell
    /// function definitions (`name() { body }`, one line or multi-line).
    pub fn run_script_text(&mut self, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let raw_line = lines[i];
            i += 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((name, rest)) = Self::function_header(line) {
                let mut body = String::new();
                let mut depth = 0i32;
                let mut collected_any_brace = false;

                for ch in rest.chars() {
                    if ch == '{' {
                        depth += 1;
                        collected_any_brace = true;
                        if depth == 1 {
                            continue;
                        }
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    if collected_any_brace {
                        body.push(ch);
                    }
                }

                while depth > 0 && i < lines.len() {
                    let next_line = lines[i];
                    i += 1;
                    for ch in next_line.chars() {
                        if ch == '{' {
                            depth += 1;
                        } else if ch == '}' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        body.push(ch);
                    }
                    body.push('\n');
                }

                self.functions.lock().unwrap().insert(name, body.trim().to_string());
                continue;
            }

            crate::run_line_with(self, line, |_| {
                if i < lines.len() {
                    let l = lines[i];
                    i += 1;
                    Some(l.to_string())
                } else {
                    None
                }
            });
        }
    }

    /// Recognizes a `name() {` (or `function name {` / `function name() {`)
    /// header, returning `(name, rest_of_line_after_open_brace_search)`.
    fn function_header(line: &str) -> Option<(String, &str)> {
        let line = line.trim();
        let (name, after) = if let Some(rest) = line.strip_prefix("function ") {
            let rest = rest.trim_start();
            let name_end = rest.find(|c: char| c.is_whitespace() || c == '(').unwrap_or(rest.len());
            let name = &rest[..name_end];
            (name, &rest[name_end..])
        } else {
            let paren = line.find("()")?;
            let name = line[..paren].trim();
            if name.is_empty()
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                return None;
            }
            (name, &line[paren + 2..])
        };

        if name.is_empty() {
            return None;
        }
        if !after.contains('{') {
            return None;
        }
        Some((name.to_string(), after))
    }

    pub fn set_positional_args(&mut self, args: Vec<String>) {
        self.positional_stack = vec![args];
    }

    /// Runs a user-defined function's body with `$1`, `$2`, ... bound to
    /// `args`, returning the function's final exit status.
    pub fn call_function(&mut self, name: &str, args: &[String]) -> i32 {
        let Some(body) = self.functions.lock().unwrap().get(name).cloned() else {
            return 127;
        };
        self.positional_stack.push(args.to_vec());
        self.run_script_text(&body);
        self.positional_stack.pop();
        self.last_exit_status
    }

    /// Returns the current function call's positional parameters, if any
    /// function call is in progress.
    fn positional_params(&self) -> Option<&Vec<String>> {
        self.positional_stack.last()
    }

    /// Looks up a shell variable, falling back to the process environment,
    /// then resolves the handful of special variables (`?`, `$`, `0`, and
    /// the positional parameters `1`.."9", `@`, `#` inside a function body).
    pub fn get_var(&self, name: &str) -> String {
        match name {
            "?" => return self.last_exit_status.to_string(),
            "$" => return std::process::id().to_string(),
            "0" => return self.arg0.clone(),
            "PWD" => {
                if let Ok(cwd) = env::current_dir() {
                    return cwd.to_string_lossy().into_owned();
                }
            }
            "OLDPWD" => {
                if let Some(ref p) = self.old_pwd {
                    return p.to_string_lossy().into_owned();
                }
            }
            "@" | "*" => {
                if let Some(params) = self.positional_params() {
                    return params.join(" ");
                }
            }
            "#" => {
                if let Some(params) = self.positional_params() {
                    return params.len().to_string();
                }
                return "0".to_string();
            }
            _ if name.len() <= 2 && name.chars().all(|c| c.is_ascii_digit()) && !name.is_empty() => {
                if let Ok(idx) = name.parse::<usize>() {
                    if idx >= 1 {
                        if let Some(params) = self.positional_params() {
                            return params.get(idx - 1).cloned().unwrap_or_default();
                        }
                        return String::new();
                    }
                }
            }
            _ => {}
        }
        if let Some(v) = self.shell_vars.lock().unwrap().get(name).cloned() {
            return v;
        }
        env::var(name).unwrap_or_default()
    }

    /// Resolves a `${NAME:+word}` / `${NAME:-word}` style parameter
    /// expansion. `op` is `+` or `-`; `word` is re-tokenized (so quotes and
    /// `$VAR` references inside it work normally) and expanded. POSIX
    /// semantics: `:-` substitutes `word` when NAME is unset/empty,
    /// otherwise keeps NAME's value; `:+` substitutes `word` when NAME is
    /// set/non-empty, otherwise expands to nothing.
    pub fn expand_param_op(&self, name: &str, op: char, word: &str) -> String {
        let current = self.get_var(name);
        let want_word = match op {
            '-' => current.is_empty(),
            '+' => !current.is_empty(),
            _ => false,
        };
        if !want_word {
            return if op == '-' { current } else { String::new() };
        }
        let tokens = crate::parser::lexer::tokenize(word);
        let parsed_word = tokens.into_iter().find_map(|t| match t {
            crate::parser::lexer::Token::Word(w) => Some(w),
            _ => None,
        });
        match parsed_word {
            Some(w) => self.expand_word_single(&w),
            None => String::new(),
        }
    }

    pub fn set_var(&mut self, name: &str, value: &str) {
        if name == "INIT_INFO" {
            self.init_info = value == "true";
        }
        self.shell_vars.lock().unwrap().insert(name.to_string(), value.to_string());
        if self.exported.contains(name) {
            unsafe {
                env::set_var(name, value);
            }
        }
    }

    pub fn export_var(&mut self, name: &str, value: Option<&str>) {
        if let Some(v) = value {
            self.shell_vars.lock().unwrap().insert(name.to_string(), v.to_string());
            unsafe {
                env::set_var(name, v);
            }
        } else if let Some(v) = self.shell_vars.lock().unwrap().get(name).cloned() {
            unsafe {
                env::set_var(name, &v);
            }
        }
        self.exported.insert(name.to_string());
    }

    pub fn unset_var(&mut self, name: &str) {
        self.shell_vars.lock().unwrap().remove(name);
        self.exported.remove(name);
        unsafe {
            env::remove_var(name);
        }
    }

    /// Detects a leading `NAME=value` assignment word (POSIX-style, no
    /// spaces around `=`). Returns `(name, value)` if `word` is a bare
    /// literal matching that shape.
    pub fn as_assignment(word: &Word) -> Option<(String, String)> {
        if word.segments.len() != 1 {
            return None;
        }
        let WordSegment::Literal(s) = &word.segments[0] else {
            return None;
        };
        let eq = s.find('=')?;
        if eq == 0 {
            return None;
        }
        let name = &s[..eq];
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
            || name.chars().next().unwrap().is_ascii_digit()
        {
            return None;
        }
        Some((name.to_string(), s[eq + 1..].to_string()))
    }

    /// Expands a single `Word` into one or more resulting strings.
    /// Quoted/single-segment words never glob or split; unquoted words with
    /// glob metacharacters expand against the filesystem.
    pub fn expand_word(&self, word: &Word) -> Vec<String> {
        let mut out = String::new();
        for seg in &word.segments {
            match seg {
                WordSegment::Literal(s) => out.push_str(s),
                WordSegment::VarExpand(name) => out.push_str(&self.get_var(name)),
                WordSegment::Tilde(s) => {
                    let home = self.home_dir.to_string_lossy();
                    if s == "~" {
                        out.push_str(&home);
                    } else if let Some(rest) = s.strip_prefix("~/") {
                        out.push_str(&home);
                        out.push('/');
                        out.push_str(rest);
                    } else {
                        out.push_str(s);
                    }
                }
                WordSegment::CommandSubst(src) => {
                    out.push_str(&self.run_command_subst(src));
                }
                WordSegment::ParamOp(name, op, w) => {
                    out.push_str(&self.expand_param_op(name, *op, w));
                }
            }
        }

        if word.quoted {
            return vec![out];
        }

        let braced = crate::utils::expand_braces(&out);
        let mut final_out = Vec::new();
        for item in braced {
            if let Some(matches) = self.try_glob(&item) {
                if !matches.is_empty() {
                    final_out.extend(matches);
                    continue;
                }
            }
            final_out.push(item);
        }
        final_out
    }

    /// Expands a `Word` into a single joined string (used where multiple
    /// results/globbing don't make sense, e.g. redirect targets).
    pub fn expand_word_single(&self, word: &Word) -> String {
        self.expand_word(word).join(" ")
    }

    fn try_glob(&self, pattern: &str) -> Option<Vec<String>> {
        if !pattern.chars().any(|c| matches!(c, '*' | '?' | '[')) {
            return None;
        }
        let mut results: Vec<String> = glob::glob(pattern)
            .ok()?
            .filter_map(|entry| entry.ok())
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        if results.is_empty() {
            return None;
        }
        results.sort();
        Some(results)
    }

    /// Runs `src` (raw source text captured from `$(...)`/backticks) as a
    /// nested script and returns its stdout with trailing newlines trimmed.
    fn run_command_subst(&self, src: &str) -> String {
        let tokens = crate::parser::lexer::tokenize(src);
        let list = crate::parser::parser::parse(tokens);
        // Reuse the current process env/shell vars by spawning through the
        // same expansion path, but capture stdout instead of inheriting it.
        let mut all_output = Vec::new();
        for (andor, _op) in &list.items {
            let expanded = self.expand_pipeline(&andor.pipeline, None);
            let output = crate::executor::pipeline::execute_capture(expanded);
            all_output.extend(output);
        }
        let mut s = String::from_utf8_lossy(&all_output).into_owned();
        while s.ends_with('\n') {
            s.pop();
        }
        s
    }

    /// Expands every word of every command in a `Pipeline` into an
    /// `ExpandedPipeline` ready for the executor. `heredoc_body` is attached
    /// to whichever command declared a heredoc redirect, if any.
    pub fn expand_pipeline(
        &self,
        pipeline: &crate::parser::Pipeline,
        heredoc_body: Option<&str>,
    ) -> crate::parser::ExpandedPipeline {
        use crate::parser::lexer::RedirectTarget;
        use crate::parser::ExpandedCommand;

        let mut commands = Vec::new();
        for cmd in &pipeline.commands {
            let mut words = self.expand_word(&cmd.program);
            for a in &cmd.args {
                words.extend(self.expand_word(a));
            }
            if words.is_empty() {
                continue;
            }

            // Resolve aliases on the program name only (first command word).
            let (program, mut rest) = {
                let mut w = words;
                let program = w.remove(0);
                (program, w)
            };
            let (program, mut rest) = self.resolve_alias(program, rest.drain(..).collect());

            let mut final_words = vec![program];
            final_words.append(&mut rest);

            let redirects: Vec<_> = cmd
                .redirects
                .iter()
                .map(|r| crate::parser::lexer::Redirect {
                    fd: r.fd,
                    append: r.append,
                    target: match &r.target {
                        RedirectTarget::File(p) => RedirectTarget::File(self.expand_str(p)),
                        RedirectTarget::Fd(n) => RedirectTarget::Fd(*n),
                        RedirectTarget::Heredoc(d) => RedirectTarget::Heredoc(d.clone()),
                        RedirectTarget::HereString(s) => {
                            RedirectTarget::HereString(self.expand_str(s))
                        }
                    },
                })
                .collect();

            let is_heredoc = redirects
                .iter()
                .any(|r| matches!(r.target, RedirectTarget::Heredoc(_)));

            let expanded_env_vars: Vec<(String, String)> = cmd
                .env_vars
                .iter()
                .map(|(k, v)| (k.clone(), self.expand_str(v)))
                .collect();

            commands.push(ExpandedCommand {
                program: final_words.remove(0),
                args: final_words,
                env_vars: expanded_env_vars,
                redirects,
                heredoc: if is_heredoc {
                    heredoc_body.map(|s| self.expand_str(s))
                } else {
                    None
                },
            });
        }

        crate::parser::ExpandedPipeline { commands }
    }

    /// Expands `$VAR`/`~` occurring in a plain string (used for redirect
    /// targets, which come from the lexer as flattened literals that may
    /// still contain `$NAME` placeholders).
    fn expand_str(&self, s: &str) -> String {
        let expanded = crate::utils::expand_env_vars_with(s, |name| self.get_var(name));
        crate::utils::expand_tilde_with(&expanded, &self.home_dir.to_string_lossy())
    }

    fn resolve_alias(&self, program: String, rest: Vec<String>) -> (String, Vec<String>) {
        let map = self.aliases.lock().unwrap();
        if let Some(alias_val) = map.get(&program) {
            let alias_words: Vec<String> = alias_val.split_whitespace().map(|s| s.to_string()).collect();
            if !alias_words.is_empty() {
                let mut new_rest = alias_words[1..].to_vec();
                new_rest.extend(rest);
                return (alias_words[0].clone(), new_rest);
            }
        }
        (program, rest)
    }

    fn get_git_branch(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["symbolic-ref", "--short", "HEAD"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() {
                return Some(branch);
            }
        }

        let mut dir = env::current_dir().ok()?;
        loop {
            let git_dir = dir.join(".git");
            if git_dir.is_dir() {
                let head_file = git_dir.join("HEAD");
                if head_file.is_file() {
                    if let Ok(content) = fs::read_to_string(head_file) {
                        let content = content.trim();
                        if content.starts_with("ref: refs/heads/") {
                            return Some(content.strip_prefix("ref: refs/heads/").unwrap().to_string());
                        } else if content.starts_with("ref: refs/tags/") {
                            return Some(content.strip_prefix("ref: refs/tags/").unwrap().to_string());
                        } else if !content.is_empty() {
                            return Some("HEAD".to_string());
                        }
                    }
                }
                break;
            }
            if !dir.pop() {
                break;
            }
        }
        None
    }

    fn get_current_dir_short(&self) -> String {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if current_dir == self.home_dir {
            return "~".to_string();
        }
        current_dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "/".to_string())
    }

    fn is_ssh(&self) -> bool {
        env::var("SSH_CLIENT").is_ok() || env::var("SSH_TTY").is_ok() || env::var("SSH_CONNECTION").is_ok()
    }

    /// Detect the current distro/OS.
    /// Returns (id, id_like, name). On Linux it reads `/etc/os-release`.
    fn detect_distro(&self) -> (String, String, String) {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = fs::read_to_string("/etc/os-release") {
                let mut id = String::new();
                let mut id_like = String::new();
                let mut name = String::new();
                for line in content.lines() {
                    if let Some(v) = line.strip_prefix("ID=") {
                        id = v.trim_matches('"').to_string();
                    } else if let Some(v) = line.strip_prefix("ID_LIKE=") {
                        id_like = v.trim_matches('"').to_string();
                    } else if let Some(v) = line.strip_prefix("NAME=") {
                        name = v.trim_matches('"').to_string();
                    }
                }
                if !id.is_empty() || !id_like.is_empty() {
                    return (id, id_like, name);
                }
            }
            return ("linux".to_string(), String::new(), "Linux".to_string());
        }
        #[cfg(target_os = "macos")]
        {
            return ("macos".to_string(), String::new(), "macOS".to_string());
        }
        #[cfg(target_os = "windows")]
        {
            return ("windows".to_string(), String::new(), "Windows".to_string());
        }
        #[allow(unreachable_code)]
        ("linux".to_string(), String::new(), "Linux".to_string())
    }

    /// Map a distro/OS id to an emoji logo.
    fn logo_for(candidate: &str) -> Option<&'static str> {
        Some(match candidate {
            "macos" => "🍎",
            "windows" => "🪟",
            // Everything else recognized here is a Linux distro id/id_like:
            // one shared penguin, since that's what actually identifies the
            // kernel/OS family the way 🍎/🪟 do for the others.
            "zorin" | "ubuntu" | "linuxmint" | "mint" | "elementary" | "pop" | "pop_os"
            | "arch" | "archarm" | "manjaro" | "endeavouros" | "endeavour" | "fedora"
            | "debian" | "raspbian" | "opensuse" | "opensuse-leap" | "opensuse-tumbleweed"
            | "gentoo" | "void" | "alpine" | "centos" | "rhel" | "kali" | "linux" => "🐧",
            _ => return None,
        })
    }

    /// Returns the OS logo (emoji) for the running system, or the value of
    /// `PROMPT_ICON` if the user has set that override.
    fn os_logo(&mut self) -> String {
        // Check for user override
        let override_icon = self.get_var("PROMPT_ICON");
        if !override_icon.is_empty() {
            return override_icon;
        }

        // Return cached logo if available (avoid repeated /etc/os-release reads)
        if let Some(ref logo) = self.cached_os_logo {
            return logo.clone();
        }

        // Detect and cache the logo
        let (id, id_like, _name) = self.detect_distro();
        let mut candidates: Vec<String> = vec![id];
        candidates.extend(id_like.split_whitespace().map(|s| s.to_string()));

        let logo = candidates
            .iter()
            .find_map(|c| Self::logo_for(c))
            .unwrap_or("🐧")
            .to_string();

        self.cached_os_logo = Some(logo.clone());
        logo
    }

    pub fn render_prompt(&mut self) -> String {
        crate::utils::emit_osc7();
        let status_part = if self.last_exit_status == 0 {
            "".to_string()
        } else {
            format!("{} {} ", "✘".bold().red(), self.last_exit_status.to_string().red())
        };

        let ssh_part = if self.is_ssh() {
            let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
            let host = env::var("HOSTNAME").unwrap_or_else(|_| "host".to_string());
            format!("{}@{} 🔐 ", user.bold().magenta(), host.bold().cyan())
        } else {
            "".to_string()
        };

        let git_part = match self.get_git_branch() {
            Some(branch) => format!(" {}", branch.green()),
            None => "".to_string(),
        };

        format!(
            "{}{}{} {} {} {} ",
            status_part,
            ssh_part,
            self.os_logo(),
            self.get_current_dir_short().bold().magenta(),
            git_part,
            ">".magenta()
        )
    }

    pub fn render_prompt_clean(&mut self) -> String {
        let status_part = if self.last_exit_status == 0 {
            "".to_string()
        } else {
            format!("✘ {} ", self.last_exit_status)
        };

        let ssh_part = if self.is_ssh() {
            let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
            let host = env::var("HOSTNAME").unwrap_or_else(|_| "host".to_string());
            format!("{}@{} 🔐 ", user, host)
        } else {
            "".to_string()
        };

        let git_part = match self.get_git_branch() {
            Some(branch) => format!(" {}", branch),
            None => "".to_string(),
        };

        format!(
            "{}{}{} {} {} > ",
            status_part,
            ssh_part,
            self.os_logo(),
            self.get_current_dir_short(),
            git_part
        )
    }
}
