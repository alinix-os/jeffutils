/// kinfo — Kernel Info (JUtils)
/// Similar to uname(1) on Linux, macOS, and Windows.
///
/// Flags:
///   -a  --all               Print all fields (same order as uname -a)
///   -s  --kernel-name       Kernel name
///   -n  --nodename          Network node name
///   -r  --kernel-release    Kernel release
///   -v  --kernel-version    Kernel version
///   -m  --machine           Machine hardware
///   -p  --processor         Processor type
///   -i  --hardware-platform Hardware platform
///   -o  --operating-system  Operating system

use std::fs;
use std::process::Command;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Helper to execute a system command and return trimmed output.
fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

// ── Individual field readers ──────────────────────────────────────────────────

fn kernel_name() -> String {
    #[cfg(target_os = "linux")]
    {
        fs::read_to_string("/proc/sys/kernel/ostype")
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Linux".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        "Darwin".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "Windows_NT".to_string()
    }
}

fn nodename() -> String {
    #[cfg(target_os = "windows")]
    {
        std::env::var("COMPUTERNAME")
            .unwrap_or_else(|_| "unknown".to_string())
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        if let Ok(name) = fs::read_to_string("/proc/sys/kernel/hostname") {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        run_cmd("hostname", &[])
            .unwrap_or_else(|| "unknown".to_string())
    }
}

fn kernel_release() -> String {
    #[cfg(target_os = "linux")]
    {
        fs::read_to_string("/proc/sys/kernel/osrelease")
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        run_cmd("sysctl", &["-n", "kern.osrelease"])
            .unwrap_or_else(|| "unknown".to_string())
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(ver) = run_cmd("cmd", &["/c", "ver"]) {
            return ver;
        }
        "unknown".to_string()
    }
}

fn kernel_version() -> String {
    #[cfg(target_os = "linux")]
    {
        fs::read_to_string("/proc/sys/kernel/version")
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        run_cmd("sysctl", &["-n", "kern.version"])
            .unwrap_or_else(|| "unknown".to_string())
    }
    #[cfg(target_os = "windows")]
    {
        "unknown".to_string()
    }
}

fn machine() -> String {
    std::env::consts::ARCH.to_string()
}

fn processor() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if line.starts_with("model name") {
                    if let Some(val) = line.split(':').nth(1) {
                        return val.trim().to_string();
                    }
                }
            }
        }
        "unknown".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        run_cmd("sysctl", &["-n", "machdep.cpu.brand_string"])
            .unwrap_or_else(|| "unknown".to_string())
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("PROCESSOR_IDENTIFIER")
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

fn hardware_platform() -> String {
    std::env::consts::ARCH.to_string()
}

/// Parse /etc/os-release to get a pretty OS name on Linux hosts.
#[cfg(target_os = "linux")]
fn parse_os_release() -> Option<String> {
    let content = fs::read_to_string("/etc/os-release").ok()?;
    let mut name = None;
    for line in content.lines() {
        if line.starts_with("PRETTY_NAME=") {
            let val = line.strip_prefix("PRETTY_NAME=")?;
            return Some(val.trim_matches('"').trim_matches('\'').to_string());
        } else if line.starts_with("NAME=") {
            let val = line.strip_prefix("NAME=")?;
            name = Some(val.trim_matches('"').trim_matches('\'').to_string());
        }
    }
    name
}

fn operating_system() -> String {
    #[cfg(target_os = "linux")]
    {
        parse_os_release().unwrap_or_else(|| "GNU/Linux".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(ver) = run_cmd("sw_vers", &["-productVersion"]) {
            format!("macOS {}", ver)
        } else {
            "macOS".to_string()
        }
    }
    #[cfg(target_os = "windows")]
    {
        "Windows".to_string()
    }
}

// ── CLI ───────────────────────────────────────────────────────────────────────

fn print_help(prog: &str) {
    println!("Usage: {prog} [OPTION]...");
    println!("Print kernel information. With no OPTION, defaults to -s.\n");
    println!("  -a, --all                Print all information, in the following order:");
    println!("                             kernel-name, nodename, kernel-release,");
    println!("                             kernel-version, machine, processor,");
    println!("                             hardware-platform, operating-system");
    println!("  -s, --kernel-name        Print the kernel name");
    println!("  -n, --nodename           Print the network node hostname");
    println!("  -r, --kernel-release     Print the kernel release");
    println!("  -v, --kernel-version     Print the kernel version");
    println!("  -m, --machine            Print the machine hardware name");
    println!("  -p, --processor          Print the processor type");
    println!("  -i, --hardware-platform  Print the hardware platform");
    println!("  -o, --operating-system   Print the operating system");
    println!("      --help               Show this help and exit");
    println!("      --version            Show version information and exit");
}

#[derive(Default)]
struct Flags {
    kernel_name:      bool,
    nodename:         bool,
    kernel_release:   bool,
    kernel_version:   bool,
    machine:          bool,
    processor:        bool,
    hardware_platform: bool,
    operating_system: bool,
}

impl Flags {
    fn enable_all(&mut self) {
        self.kernel_name       = true;
        self.nodename          = true;
        self.kernel_release    = true;
        self.kernel_version    = true;
        self.machine           = true;
        self.processor         = true;
        self.hardware_platform = true;
        self.operating_system  = true;
    }

    fn none_set(&self) -> bool {
        !self.kernel_name
            && !self.nodename
            && !self.kernel_release
            && !self.kernel_version
            && !self.machine
            && !self.processor
            && !self.hardware_platform
            && !self.operating_system
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("kinfo", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = std::env::args().collect();
    let prog = args.first().map(String::as_str).unwrap_or("kinfo");

    let mut flags = Flags::default();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--help" => {
                print_help(prog);
                return;
            }
            "--version" => {
                println!("kinfo (JUtils) 0.1.0");
                return;
            }
            "--all" => flags.enable_all(),
            "--kernel-name"       => flags.kernel_name       = true,
            "--nodename"          => flags.nodename           = true,
            "--kernel-release"    => flags.kernel_release     = true,
            "--kernel-version"    => flags.kernel_version     = true,
            "--machine"           => flags.machine            = true,
            "--processor"         => flags.processor          = true,
            "--hardware-platform" => flags.hardware_platform  = true,
            "--operating-system"  => flags.operating_system   = true,

            // Short flags — may be combined: -snrvm etc.
            s if s.starts_with('-') && s.len() > 1 && !s.starts_with("--") => {
                for ch in s.chars().skip(1) {
                    match ch {
                        'a' => flags.enable_all(),
                        's' => flags.kernel_name       = true,
                        'n' => flags.nodename           = true,
                        'r' => flags.kernel_release     = true,
                        'v' => flags.kernel_version     = true,
                        'm' => flags.machine            = true,
                        'p' => flags.processor          = true,
                        'i' => flags.hardware_platform  = true,
                        'o' => flags.operating_system   = true,
                        c => {
                            eprintln!("{prog}: invalid option -- '{c}'");
                            eprintln!("Try '{prog} --help' for more information.");
                            std::process::exit(1);
                        }
                    }
                }
            }

            other => {
                eprintln!("{prog}: unrecognized option '{other}'");
                eprintln!("Try '{prog} --help' for more information.");
                std::process::exit(1);
            }
        }
    }

    // Default: just kernel name (same as uname with no args → -s)
    if flags.none_set() {
        flags.kernel_name = true;
    }

    // Build output — same field order as uname -a
    let mut parts: Vec<String> = Vec::new();

    if flags.kernel_name       { parts.push(kernel_name()); }
    if flags.nodename          { parts.push(nodename()); }
    if flags.kernel_release    { parts.push(kernel_release()); }
    if flags.kernel_version    { parts.push(kernel_version()); }
    if flags.machine           { parts.push(machine()); }
    if flags.processor         { parts.push(processor()); }
    if flags.hardware_platform { parts.push(hardware_platform()); }
    if flags.operating_system  { parts.push(operating_system()); }

    println!("{}", parts.join(" "));
}
