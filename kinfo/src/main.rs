/// kinfo — Kernel Info (JUtils)
/// Similar to uname(1) on Linux, but reads OS identity from /Sys/os-info
/// as per the JCore File Hierarchy Standard (JFHS).
///
/// Flags:
///   -a  --all               Print all fields (same order as uname -a)
///   -s  --kernel-name       Kernel name        (/Sys/kernel/name)
///   -n  --nodename          Network node name  (/Sys/kernel/hostname)
///   -r  --kernel-release    Kernel release     (/Sys/kernel/release)
///   -v  --kernel-version    Kernel version     (/Sys/kernel/version)
///   -m  --machine           Machine hardware   (/Sys/kernel/arch)
///   -p  --processor         Processor type     (/Sys/smp/cpu0 → cpu type)
///   -i  --hardware-platform Hardware platform  (/Sys/kernel/platform)
///   -o  --operating-system  Operating system   (/Sys/os-info)

use std::fs;

// ── JFHS paths ────────────────────────────────────────────────────────────────
const SYS_KERNEL_NAME:     &str = "/Sys/kernel/name";
const SYS_KERNEL_HOSTNAME: &str = "/Sys/kernel/hostname";
const SYS_KERNEL_RELEASE:  &str = "/Sys/kernel/release";
const SYS_KERNEL_VERSION:  &str = "/Sys/kernel/version";
const SYS_KERNEL_ARCH:     &str = "/Sys/kernel/arch";
const SYS_KERNEL_PLATFORM: &str = "/Sys/kernel/platform";
const SYS_CPU0:            &str = "/Sys/smp/cpu0";
const SYS_OS_INFO:         &str = "/Sys/os-info";

// ── Linux /proc fallbacks (used during host-Linux development) ────────────────
const PROC_OSTYPE:   &str = "/proc/sys/kernel/ostype";
const PROC_HOSTNAME: &str = "/proc/sys/kernel/hostname";
const PROC_RELEASE:  &str = "/proc/sys/kernel/osrelease";
const PROC_VERSION:  &str = "/proc/sys/kernel/version";
const PROC_CPUINFO:  &str = "/proc/cpuinfo";

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Read a one-line virtual file, trim whitespace.
fn read_sys(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Try JFHS path first, then Linux fallback, then a hard-coded default.
fn read_or(jfhs: &str, fallback: &str, default: &str) -> String {
    read_sys(jfhs)
        .or_else(|| read_sys(fallback))
        .unwrap_or_else(|| default.to_string())
}

// ── Individual field readers ──────────────────────────────────────────────────

fn kernel_name() -> String {
    read_or(SYS_KERNEL_NAME, PROC_OSTYPE, "JCore")
}

fn nodename() -> String {
    read_or(SYS_KERNEL_HOSTNAME, PROC_HOSTNAME, "unknown")
}

fn kernel_release() -> String {
    read_or(SYS_KERNEL_RELEASE, PROC_RELEASE, "unknown")
}

fn kernel_version() -> String {
    read_or(SYS_KERNEL_VERSION, PROC_VERSION, "unknown")
}

fn machine() -> String {
    read_sys(SYS_KERNEL_ARCH)
        .unwrap_or_else(|| std::env::consts::ARCH.to_string())
}

fn processor() -> String {
    // JFHS: /Sys/smp/cpu0 contains lines like "vendor_id: JCore"
    if let Some(content) = read_sys(SYS_CPU0) {
        for line in content.lines() {
            if line.starts_with("vendor_id") {
                if let Some(val) = line.split(':').nth(1) {
                    return val.trim().to_string();
                }
            }
        }
    }
    // Linux fallback: /proc/cpuinfo "model name"
    if let Ok(content) = fs::read_to_string(PROC_CPUINFO) {
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

fn hardware_platform() -> String {
    read_sys(SYS_KERNEL_PLATFORM)
        .unwrap_or_else(|| std::env::consts::ARCH.to_string())
}

/// Operating system — reads /Sys/os-info (JFHS), falls back to "Corix".
fn operating_system() -> String {
    read_sys(SYS_OS_INFO).unwrap_or_else(|| "Corix".to_string())
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
    println!("  -o, --operating-system   Print the operating system (reads /Sys/os-info)");
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
            s if s.starts_with('-') && !s.starts_with("--") => {
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
