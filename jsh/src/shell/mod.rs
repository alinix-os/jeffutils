use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crossterm::style::{Color, Stylize};

pub struct ShellState {
    pub last_exit_status: i32,
    pub home_dir: PathBuf,
    pub init_info: bool,
    pub aliases: Arc<Mutex<HashMap<String, String>>>,
    pub old_pwd: Option<PathBuf>,
}

impl ShellState {
    pub fn new() -> Self {
        let home = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        
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

        Self {
            last_exit_status: 0,
            home_dir: home,
            init_info: true,
            aliases: aliases_map,
            old_pwd: None,
        }
    }

    pub fn load_jshrc(&mut self) {
        let jshrc_path = self.home_dir.join(".jshrc");
        if !jshrc_path.exists() {
            let default_jshrc = "\
# jsh configuration file
INIT_INFO=true

alias c=\"clear\"
alias ls=\"ls --color=auto\"
alias grep=\"grep --color=auto\"

# Custom Exports
export EDITOR=texit
export PATH=/bin:/usr/bin:/usr/local/bin
";
            let _ = fs::write(&jshrc_path, default_jshrc);
        }

        if let Ok(content) = fs::read_to_string(&jshrc_path) {
            let mut map = self.aliases.lock().unwrap();
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                if line.starts_with("INIT_INFO=") {
                    let val = line.strip_prefix("INIT_INFO=").unwrap_or("true").trim();
                    self.init_info = val == "true";
                } else if line.starts_with("export ") {
                    let expr = line.strip_prefix("export ").unwrap_or("").trim();
                    let parts: Vec<&str> = expr.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let val = parts[1].trim().trim_matches('"').trim_matches('\'');
                        unsafe {
                            env::set_var(key, val);
                        }
                    }
                } else if line.starts_with("alias ") {
                    let expr = line.strip_prefix("alias ").unwrap_or("").trim();
                    let parts: Vec<&str> = expr.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let val = parts[1].trim().trim_matches('"').trim_matches('\'');
                        map.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
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

    /// Map a distro id to a single-glyph logo and a brand-ish color.
    fn logo_for(candidate: &str) -> Option<(char, Color)> {
        Some(match candidate {
            "zorin" => ('Z', Color::Magenta),
            "ubuntu" => ('U', Color::Rgb { r: 228, g: 76, b: 23 }),
            "linuxmint" | "mint" => ('M', Color::Green),
            "elementary" => ('e', Color::Blue),
            "pop" | "pop_os" => ('P', Color::Cyan),
            "arch" | "archarm" => ('A', Color::Cyan),
            "manjaro" => ('M', Color::Green),
            "endeavouros" | "endeavour" => ('E', Color::Cyan),
            "fedora" => ('F', Color::Blue),
            "debian" => ('D', Color::Red),
            "raspbian" => ('R', Color::Red),
            "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => ('S', Color::Green),
            "gentoo" => ('G', Color::Cyan),
            "void" => ('V', Color::Green),
            "alpine" => ('A', Color::Cyan),
            "centos" => ('C', Color::Yellow),
            "rhel" => ('R', Color::Red),
            "kali" => ('K', Color::Blue),
            "macos" => ('M', Color::White),
            "windows" => ('W', Color::Cyan),
            _ => return None,
        })
    }

    /// Returns the OS logo glyph (styled) for the running system.
    fn os_logo(&self) -> String {
        let (id, id_like, _name) = self.detect_distro();
        let mut candidates: Vec<String> = vec![id];
        candidates.extend(id_like.split_whitespace().map(|s| s.to_string()));

        let (glyph, color) = candidates
            .iter()
            .find_map(|c| Self::logo_for(c))
            .unwrap_or(('L', Color::Yellow));

        glyph.to_string().with(color).bold().to_string()
    }

    pub fn render_prompt(&self) -> String {
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

    pub fn process_args(&self, raw_args: &[String]) -> Vec<String> {
        if raw_args.is_empty() {
            return vec![];
        }

        let mut cmd = raw_args[0].clone();
        let mut rest = raw_args[1..].to_vec();

        // Resolve aliases
        {
            let map = self.aliases.lock().unwrap();
            if let Some(alias_val) = map.get(&cmd) {
                let alias_words: Vec<String> = alias_val.split_whitespace().map(|s| s.to_string()).collect();
                if !alias_words.is_empty() {
                    cmd = alias_words[0].clone();
                    let mut new_rest = alias_words[1..].to_vec();
                    new_rest.extend(rest);
                    rest = new_rest;
                }
            }
        }

        let mut final_args = vec![cmd];
        final_args.extend(rest);

        // Expand environment variables
        for arg in final_args.iter_mut() {
            *arg = expand_env_vars(arg);
        }

        // Expand tilde ~
        let home_str = self.home_dir.to_string_lossy().into_owned();
        for arg in final_args.iter_mut() {
            if arg == "~" {
                *arg = home_str.clone();
            } else if arg.starts_with("~/") {
                *arg = arg.replacen("~/", &format!("{}/", home_str), 1);
            }
        }

        final_args
    }
}
use crate::utils::expand_env_vars;
