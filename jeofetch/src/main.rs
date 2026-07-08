use std::env;
use crossterm::style::{Color, Stylize};
use sysinfo::System;

/// Fallback ASCII art logo "JA" displayed by jeofetch when the OS is unknown.
const JA_LOGO: &[&str] = &[
    r"     ██╗ █████╗ ",
    r"     ██║██╔══██╗",
    r"     ██║███████║",
    r"██   ██║██╔══██║",
    r"╚█████╔╝██║  ██║",
    r" ╚════╝ ╚═╝  ╚═╝",
];

/// Arch Linux logo.
const ARCH_LOGO: &[&str] = &[
    r"                   -",
    r"                  .o+",
    r"                 `ooo/",
    r"                `+oooo:",
    r"               `+ooooo:",
    r"               -+ooooo+:",
    r"             `/:-:++oooo+:",
    r"            `/++++/+++++++:",
    r"           `/++++++++++++++:",
    r"          `/+++ooooooooooooo/`",
    r"         ./ooosssso++osssssso+`",
    r"        .oossssso-````/ossssss+`",
    r"       -osssssso.      :ssssssso.",
    r"      :osssssss/        osssso++.",
    r"     /ossssssss/        +ssssooo/-",
    r"   `/ossssso+/:-        -:/+osssso+-",
    r"  `+sso+:-`                 `.-/+oso:",
    r" `++:.                           `-/+/.",
    r" .`                                 `/",
];

/// Ubuntu logo.
const UBUNTU_LOGO: &[&str] = &[
    r"            .-/+oossssoo+/-.",
    r"        `:+ssssssssssssssssss+:",
    r"      -+ssssssssssssssssssyyssss+-",
    r"    .ossssssssssssssssssdMMMNysssso.",
    r"   /ssssssssssshdmmNNmmyNMMMMhssssss/",
    r"  +ssssssssshmydMMMMMMMNddddyssssssss+",
    r" /sssssssshNMMMyhhyyyyhmNMMMNhssssssss/",
    r".ssssssssdMMMNhsssssssssshNMMMdssssssss.",
    r"+sssshhhyNMMNyssssssssssssyNMMMysssssss+",
    r"ossyNMMMNyMMhsssssssssssssshmmmhssssssso",
    r"ossyNMMMNyMMhsssssssssssssshmmmhssssssso",
    r"+sssshhhyNMMNyssssssssssssyNMMMysssssss+",
    r".ssssssssdMMMNhsssssssssshNMMMdssssssss.",
    r" /sssssssshNMMMyhhyyyyhdNMMMNhssssssss/",
    r"  +sssssssssdmydMMMMMMMMddddyssssssss+",
    r"   /ssssssssssshdmNNNNmyNMMMMhssssss/",
    r"    .ossssssssssssssssssdMMMNysssso.",
    r"      -+sssssssssssssssssyyyssss+-",
    r"        `:+ssssssssssssssssss+:`",
    r"            .-/+oossssoo+/-.",
];

/// Zorin OS logo.
const ZORIN_LOGO: &[&str] = &[
    r"    .-----------.",
    r"   /  ZZZZZZZZ   \",
    r"  |  Z       Z   |",
    r"  |  Z       Z   |",
    r"  |  Z      Z    |",
    r"  |  Z     Z     |",
    r"  |  Z    Z      |",
    r"  |  Z   Z       |",
    r"  |  Z  Z        |",
    r"  |  Z Z         |",
    r"  |  ZZ          |",
    r"  |  Z           |",
    r"   \             /",
    r"    '-----------'",
];


/// Debian logo.
const DEBIAN_LOGO: &[&str] = &[
    r"        _,met$$$$$gg.",
    r"     ,g$$$$$$$$$$$$$$$P.",
    r#"   ,g$$P""       """Y$$"."#,
    r"  ,$$P'              `$$$.",
    r"',$$P       ,ggs.     `$$b:",
    r#"`d$'     ,$P\"'   .    $$$"#,
    r" $$P      d$'     ,    $$P",
    r" $$:      $$.   -    ,d$$'",
    r" $$;      Y$b._   _,d$P'",
    r#" Y$$.    `.`\"Y$$$$P\"'"#,
    r#" `$$b      \"-.__"#,
    r"  `Y$$",
    r"   `Y$$",
    r"    `Y$$",
    r"     `$$b",
    r"      `Y$$",
    r"       `Y$$",
    r"        `$$",
    r"         `$",
];


/// A logo together with the brand color it should be rendered in.
struct Logo {
    lines: &'static [&'static str],
    color: Color,
}

/// The "JA" logo used as the fallback when the OS is unknown.
fn ja_logo() -> Logo {
    Logo {
        lines: JA_LOGO,
        color: Color::Magenta,
    }
}

/// Returns the distribution id (lowercased), preferring /etc/os-release and
/// falling back to sysinfo's OS name.
fn distro_id() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("ID=") {
                return rest.trim_matches('"').to_lowercase();
            }
        }
    }
    System::name().unwrap_or_default().to_lowercase()
}

/// Picks the ASCII logo for the current OS. Falls back to the "JA" logo when
/// the OS/distro is not recognized.
fn select_logo() -> Logo {
    match distro_id().as_str() {
        "arch" | "archlinux" | "manjaro" | "endeavouros" | "artix" => Logo {
            lines: ARCH_LOGO,
            color: Color::Cyan,
        },
        "zorin" => Logo {
            lines: ZORIN_LOGO,
            color: Color::Rgb { r: 0x6E, g: 0x1A, b: 0xD1 },
        },
        "ubuntu" | "linuxmint" | "pop" | "elementary" | "kali" | "raspbian" => Logo {
            lines: UBUNTU_LOGO,
            color: Color::Rgb { r: 0xE9, g: 0x54, b: 0x20 },
        },
        "debian" => Logo {
            lines: DEBIAN_LOGO,
            color: Color::Red,
        },
        "fedora" | "centos" | "rhel" | "almalinux" | "rocky" => Logo {
            lines: FEDORA_LOGO,
            color: Color::Blue,
        },
        "linux" => Logo {
            lines: LINUX_LOGO,
            color: Color::Yellow,
        },
        "darwin" | "macos" => Logo {
            lines: MACOS_LOGO,
            color: Color::White,
        },
        "windows" => Logo {
            lines: WINDOWS_LOGO,
            color: Color::Blue,
        },
        _ => ja_logo(),
    }
}


/// Fedora logo.
const FEDORA_LOGO: &[&str] = &[
    r"           /:-------------:\",
    r"        :-------------------::",
    r"      :-----------/shhOHbmp---:",
    r"    /-----------omMMMNNNMMD  ---:",
    r"   :-----------sMMMMNMNMP.    ---:",
    r"  :-----------:MMMdP-------    ---\",
    r" ,------------:MMMd--------    ---\",
    r" :-----------:MMMd-------    -----\",
    r" :---------oNMMMMMP------      ---\",
    r" :-------dMMMMMMMMP-----       ---\",
    r" :------:sdNMMMNMP-------      ---\",
    r" :-----:sdNMMMNMP-------       ---\",
    r" :----:sdNMMMNMP--------      ---\",
    r" :---:sdNMMMNMP---------      ---\",
    r" :--:sdNMMMNMP-----------     ---\",
    r" :--:sdNMMMNMP-----------    ---\",
    r" :-+MMMMMNMP-------------    ---\",
    r"  .+MMMMMNMP-------------    --\",
    r"    .+MMMMMNMP-----------    --\",
    r"      `:://:--------------`--\",
];

/// Generic Linux penguin logo (fallback for unrecognized Linux distros).
const LINUX_LOGO: &[&str] = &[
    r"       .NNNNNNN.",
    r"      .NNNNNNNNN.",
    r"      NNNNNNNNNN.",
    r"      NNNNNNNNNN.",
    r"      `NNNNNNNNN'",
    r"       `NNNNNNN'",
    r"        `NNNNN'",
    r"         `NNN'",
    r"          `N'",
];

/// macOS logo.
const MACOS_LOGO: &[&str] = &[
    r"                 `:;;;,`",
    r"       ;;;;;;;;;;;;",
    r"    :;;;;;;;;;;;;;;;:",
    r"  ;;:;;:;;;;;;;;;;;;;;:;;",
    r"    ;; ;;;;;;;;;;;;;;;;",
    r"       :;;;;;;;;;;;;;;;",
    r"   ;;;;;;;;;;;;;;;;;;;;",
    r"  ;; ;;;;;;;;;;;;;;;;;",
    r"     :;;;;;;;;;;;;;;;;",
    r"        ;;;;;;;;;;;;;;",
    r"          ;;;;;;;;;;",
    r"            :;;;;;",
];

/// Windows logo.
const WINDOWS_LOGO: &[&str] = &[
    r"           ,.=:!!t3Z3z.,",
    r"          :tt:::tt333EE3",
    r"         Et:::ztt33EEEL @Ee.,..,",
    r"        ;tt:::tt333EE7 ;EEEEEEttt:::tEe.",
    r"       :Et:::zt333EEQ. $EEEEEtttt:::t33.",
    r"      :t0:::tt333EEF @EEEEEEttttt:::t#;",
    r"      :t0:::tt333EEF ;EEEEEEttttt:::t#;",
    r"      :t0:::tt333EEF ;EEEEEEttttt:::t#;",
    r"      :t0:::tt333EEF ;EEEEEEttttt:::t#;",
    r"       :0t:::tt333EEF ;EEEEEEttttt::te",
    r"        '0t:::tt333EEF ;EEEEEEttttt::t",
    r"         '0t:::tt333EEF ;EEEEEEttttt::t",
    r"          '0t:::tt333EEF ;EEEEEEttttt::t",
    r"           '0t:::tt333EEF ;EEEEEEttttt::t",
];

fn main() {
    run_jeofetch();
}

fn run_jeofetch() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let uptime_sec = System::uptime();
    let up_hours = uptime_sec / 3600;
    let up_mins = (uptime_sec % 3600) / 60;
    let hostname = System::host_name().unwrap_or_else(|| "localhost".to_string());
    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let shell = "jsh";

    let cpu = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let total_mem = sys.total_memory() / 1024 / 1024;
    let used_mem = sys.used_memory() / 1024 / 1024;

    // Info lines paired with logo lines
    let info_lines: Vec<String> = vec![
        format!(
            "{}@{}",
            user.bold().green(),
            hostname.bold().green()
        ),
        format!("{:<10} {}", "OS:".cyan(), os_name),
        format!("{:<10} {}", "Kernel:".cyan(), kernel),
        format!("{:<10} {}h {}m", "Uptime:".cyan(), up_hours, up_mins),
        format!("{:<10} {}", "Shell:".cyan(), shell),
        format!("{:<10} {}", "CPU:".cyan(), cpu),
        format!(
            "{:<10} {} / {} MB",
            "Memory:".cyan(),
            used_mem,
            total_mem
        ),
    ];

    let logo = select_logo();
    let logo_width = logo.lines.iter().map(|l| l.len()).max().unwrap_or(0);

    // Print each logo line alongside the info
    for (i, logo_line) in logo.lines.iter().enumerate() {
        let colored_logo = logo_line.with(logo.color).bold().to_string();
        if let Some(info) = info_lines.get(i) {
            println!("{}  {}", colored_logo, info);
        } else {
            println!("{}", colored_logo);
        }
    }

    // Print any remaining info lines that exceed logo height
    for info in info_lines.iter().skip(logo.lines.len()) {
        println!("{:width$}  {}", "", info, width = logo_width);
    }

    println!();
}
