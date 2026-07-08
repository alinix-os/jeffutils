use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crossterm::style::Stylize;

// Embed the AlinixLogo font file bytes directly in the binary
const FONT_BYTES: &[u8] = include_bytes!("/home/jefferson/Desktop/projects/Alinix/Alinix-deb/assets/AlinixLogo-Regular.otf");

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

    pub fn ensure_font_installed(&self) {
        #[cfg(unix)]
        {
            let fonts_dir = self.home_dir.join(".local/share/fonts");
            let target_font = fonts_dir.join("AlinixLogo-Regular.otf");
            let mut cache_updated = false;

            if !target_font.exists() {
                let _ = fs::create_dir_all(&fonts_dir);
                if fs::write(&target_font, FONT_BYTES).is_ok() {
                    cache_updated = true;
                }
            }

            let fontconfig_dir = self.home_dir.join(".config/fontconfig");
            let fonts_conf = fontconfig_dir.join("fonts.conf");
            if !fonts_conf.exists() {
                let _ = fs::create_dir_all(&fontconfig_dir);
                let fallback_xml = "\
<?xml version=\"1.0\"?>
<!DOCTYPE fontconfig SYSTEM \"urn:fontconfig:fonts.dtd\">
<fontconfig>
    <!-- Fallback AlinixLogo for monospace fonts -->
    <match target=\"pattern\">
        <test qual=\"any\" name=\"family\">
            <string>monospace</string>
        </test>
        <edit name=\"family\" mode=\"append\" binding=\"strong\">
            <string>AlinixLogo</string>
        </edit>
    </match>
</fontconfig>
";
                if fs::write(&fonts_conf, fallback_xml).is_ok() {
                    cache_updated = true;
                }
            }

            if cache_updated {
                let _ = Command::new("fc-cache")
                    .arg("-f")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
        #[cfg(target_os = "macos")]
        {
            let fonts_dir = self.home_dir.join("Library/Fonts");
            let target_font = fonts_dir.join("AlinixLogo-Regular.otf");
            if !target_font.exists() {
                let _ = fs::write(&target_font, FONT_BYTES);
            }
        }
        #[cfg(windows)]
        {
            if let Some(local_appdata) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
                let fonts_dir = local_appdata.join("Microsoft\\Windows\\Fonts");
                let target_font = fonts_dir.join("AlinixLogo-Regular.otf");
                if !target_font.exists() {
                    let _ = fs::create_dir_all(&fonts_dir);
                    let _ = fs::write(&target_font, FONT_BYTES);
                }
            }
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

        let logo = '\u{e000}';
        
        let git_part = match self.get_git_branch() {
            Some(branch) => format!(" {}", branch.green()),
            None => "".to_string(),
        };

        format!(
            "{}{}{} {} {} {} ",
            status_part,
            ssh_part,
            logo.to_string().bold().magenta(),
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
