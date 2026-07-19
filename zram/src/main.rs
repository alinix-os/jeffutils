fn print_usage() {
    let program_name = std::env::args().nth(0).unwrap_or_else(|| "zram".into());
    eprintln!("Usage: {} <subcommand> [args]", program_name);
    eprintln!();
    eprintln!("Subcommands:");
    eprintln!("  status               Show ZRAM device status and compression info");
    eprintln!("  enable [size]        Enable ZRAM device (e.g. 20GB, 1024MB, 2G). Default size: 1024MB");
    eprintln!("  disable              Disable and reset ZRAM devices");
    eprintln!("  configure            Configure ZRAM settings (e.g. algorithm, max_comp_streams)");
    eprintln!("  help [subcommand]    Show this help, or help for a specific subcommand");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help           Show general help message");
    eprintln!("  -v, --version        Show version information");
}

fn print_subcommand_help(subcommand: &str) {
    match subcommand {
        "status" => {
            println!("Usage: zram status");
            println!();
            println!("Displays the current status of all initialized ZRAM devices.");
            println!("Shows disk size, compression algorithm, original and compressed sizes,");
            println!("and the calculated compression ratio.");
        }
        "enable" => {
            println!("Usage: zram enable [size]");
            println!();
            println!("Enables ZRAM on /dev/zram0 with the specified size.");
            println!("The size can be specified with optional units: B, KB, MB, GB (or K, M, G).");
            println!("If no unit is specified, MB is assumed.");
            println!();
            println!("Examples:");
            println!("  zram enable 20GB      # Enables ZRAM with 20 GB");
            println!("  zram enable 512M      # Enables ZRAM with 512 MB");
            println!("  zram enable 2048      # Enables ZRAM with 2048 MB (default unit is MB)");
            println!();
            println!("Note: This command usually requires root privileges (sudo).");
        }
        "disable" => {
            println!("Usage: zram disable");
            println!();
            println!("Disables all active ZRAM devices and resets their disk sizes to 0.");
            println!();
            println!("Note: This command usually requires root privileges (sudo).");
        }
        "configure" => {
            println!("Usage: zram configure [options]");
            println!();
            println!("Configure ZRAM parameters such as the compression algorithm.");
            println!("Note: This command is currently under development.");
        }
        _ => {
            eprintln!("Unknown subcommand: '{}'", subcommand);
            print_usage();
        }
    }
}

#[cfg(target_os = "linux")]
fn read_sysfs(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

#[cfg(target_os = "linux")]
fn write_sysfs(path: &str, value: &str) -> Result<(), String> {
    std::fs::write(path, value).map_err(|e| format!("{}", e))
}

#[cfg(target_os = "linux")]
fn show_status() {
    println!("ZRAM Status:");
    let base = "/sys/block";
    let dir = match std::fs::read_dir(base) {
        Ok(d) => d,
        Err(_) => {
            println!("  ZRAM not available or not supported");
            return;
        }
    };

    let mut found = false;
    for entry in dir.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("zram") {
            continue;
        }
        found = true;

        let dev_path = entry.path();
        let dev_name = format!("/dev/{}", name_str);

        let disksize = read_sysfs(&format!("{}/disksize", dev_path.display()))
            .map(|s| format_size(s.parse::<u64>().unwrap_or(0)));
        let comp_algorithm = read_sysfs(&format!("{}/comp_algorithm", dev_path.display()))
            .unwrap_or_else(|| "unknown".into());
        let orig_size = read_sysfs(&format!("{}/orig_data_size", dev_path.display()))
            .map(|s| format_size(s.parse::<u64>().unwrap_or(0)));
        let comp_size = read_sysfs(&format!("{}/compr_data_size", dev_path.display()))
            .map(|s| format_size(s.parse::<u64>().unwrap_or(0)));

        println!("  {}:", dev_name);
        println!("    Device       : {}", dev_name);
        if let Some(size) = disksize {
            println!("    Disk Size    : {}", size);
        }
        println!("    Algorithm    : {}", comp_algorithm);
        if let Some(orig) = orig_size {
            println!("    Original     : {}", orig);
        }
        if let Some(comp) = comp_size {
            println!("    Compressed   : {}", comp);
        }
        if let (Some(orig_val), Some(comp_val)) = (
            read_sysfs(&format!("{}/orig_data_size", dev_path.display()))
                .and_then(|s| s.parse::<u64>().ok()),
            read_sysfs(&format!("{}/compr_data_size", dev_path.display()))
                .and_then(|s| s.parse::<u64>().ok()),
        ) {
            if comp_val > 0 {
                let ratio = (orig_val as f64 / comp_val as f64) * 100.0;
                println!("    Compression  : {:.1}%", ratio);
            }
        }
    }

    if !found {
        println!("  No ZRAM devices found");
    }
}

#[cfg(not(target_os = "linux"))]
fn show_status() {
    println!("ZRAM is only available on Linux");
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim().to_uppercase();
    if s.is_empty() {
        return Err("Size cannot be empty".into());
    }

    let mut num_str = String::new();
    let mut unit_str = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() || c == '.' {
            num_str.push(c);
        } else if c.is_alphabetic() {
            unit_str.push(c);
        } else if !c.is_whitespace() {
            return Err(format!("Invalid character '{}' in size", c));
        }
    }

    let val: f64 = num_str.parse().map_err(|_| format!("Invalid number format '{}'", num_str))?;

    let bytes = match unit_str.as_str() {
        "" | "M" | "MB" => (val * 1024.0 * 1024.0) as u64,
        "K" | "KB" => (val * 1024.0) as u64,
        "G" | "GB" => (val * 1024.0 * 1024.0 * 1024.0) as u64,
        "B" => val as u64,
        _ => return Err(format!("Unknown unit '{}'. Supported units: B, KB, MB, GB (or K, M, G). Default is MB if omitted.", unit_str)),
    };

    Ok(bytes)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    // Check for global flags first
    if args.len() == 1 {
        if args[0] == "-h" || args[0] == "--help" {
            print_usage();
            return;
        }
        if args[0] == "-v" || args[0] == "--version" {
            println!("zram version 0.1.0");
            return;
        }
    }

    let action = args[0].as_str();

    match action {
        "help" => {
            if args.len() > 1 {
                print_subcommand_help(&args[1]);
            } else {
                print_usage();
            }
            return;
        }
        "status" => {
            if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
                print_subcommand_help("status");
                return;
            }
            show_status();
        }
        "enable" => {
            if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
                print_subcommand_help("enable");
                return;
            }

            #[cfg(target_os = "linux")]
            {
                let size_arg = args.get(1).map(|s| s.as_str()).unwrap_or("1024");
                let size_bytes = match parse_size(size_arg) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        eprintln!("Error: invalid size: {}", e);
                        std::process::exit(1);
                    }
                };

                println!("Enabling ZRAM with {}...", format_size(size_bytes));

                // Try to load kernel module if needed, or check if /sys/block/zram0 exists
                if !std::path::Path::new("/sys/block/zram0").exists() {
                    eprintln!("Warning: /sys/block/zram0 not found. Attempting to load 'zram' kernel module...");
                    let modprobe_success = std::process::Command::new("modprobe")
                        .arg("zram")
                        .status()
                        .map(|status| status.success())
                        .unwrap_or(false);

                    if !modprobe_success {
                        eprintln!("Error: ZRAM device '/sys/block/zram0' not found and could not load 'zram' kernel module.");
                        eprintln!("Please load the module manually (e.g., 'sudo modprobe zram') or run this command with 'sudo'.");
                        std::process::exit(1);
                    }
                    if !std::path::Path::new("/sys/block/zram0").exists() {
                        eprintln!("Error: 'modprobe zram' succeeded, but '/sys/block/zram0' still does not exist.");
                        std::process::exit(1);
                    }
                }

                // If zram0 exists, we might need to reset first. If it's already active swap, we should turn it off.
                let _ = std::process::Command::new("swapoff").arg("/dev/zram0").status();
                if let Err(e) = write_sysfs("/sys/block/zram0/reset", "1") {
                    if e.contains("Permission denied") || e.contains("os error 13") {
                        eprintln!("Error: Permission denied. Please run this command as root or with sudo.");
                        std::process::exit(1);
                    }
                }

                match write_sysfs("/sys/block/zram0/disksize", &size_bytes.to_string()) {
                    Ok(_) => {
                        println!("ZRAM device /dev/zram0 configured with {}", format_size(size_bytes));
                        
                        // Format as swap
                        println!("Formatting /dev/zram0 as swap...");
                        let mkswap_status = std::process::Command::new("mkswap")
                            .arg("/dev/zram0")
                            .status();
                        
                        match mkswap_status {
                            Ok(status) if status.success() => {
                                // Activate swap with high priority (e.g. 100) so it takes precedence over disk swap
                                println!("Activating swap on /dev/zram0...");
                                let swapon_status = std::process::Command::new("swapon")
                                    .arg("-p")
                                    .arg("100")
                                    .arg("/dev/zram0")
                                    .status();
                                
                                match swapon_status {
                                    Ok(status) if status.success() => {
                                        println!("ZRAM swap successfully enabled and activated!");
                                    }
                                    _ => {
                                        eprintln!("Error: could not activate swap (swapon failed).");
                                        std::process::exit(1);
                                    }
                                }
                            }
                            _ => {
                                eprintln!("Error: could not format /dev/zram0 as swap (mkswap failed).");
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        if e.contains("Permission denied") || e.contains("os error 13") {
                            eprintln!("Error: Permission denied. Please run this command as root or with sudo.");
                        } else if e.contains("No such file or directory") || e.contains("os error 2") {
                            eprintln!("Error: could not enable ZRAM. The ZRAM module might not be loaded.");
                            eprintln!("Try running: sudo modprobe zram");
                        } else {
                            eprintln!("Error: could not enable ZRAM: {}", e);
                        }
                        std::process::exit(1);
                    }
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                eprintln!("Error: ZRAM is only available on Linux");
                std::process::exit(1);
            }
        }
        "disable" => {
            if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
                print_subcommand_help("disable");
                return;
            }
            #[cfg(target_os = "linux")]
            {
                println!("Disabling all ZRAM devices...");
                let zram_devices: Vec<String> = std::fs::read_dir("/sys/block/")
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .filter(|name| name.starts_with("zram"))
                    .collect();

                if zram_devices.is_empty() {
                    println!("No ZRAM devices found");
                    return;
                }

                for dev in &zram_devices {
                    let dev_path = format!("/dev/{}", dev);
                    let reset_path = format!("/sys/block/{}/reset", dev);
                    let _ = std::process::Command::new("swapoff").arg(&dev_path).status();
                    match write_sysfs(&reset_path, "1") {
                        Ok(_) => println!("  {} disabled and reset", dev_path),
                        Err(e) => {
                            if e.contains("Permission denied") || e.contains("os error 13") {
                                eprintln!("Error: Permission denied. Please run this command as root or with sudo.");
                            } else {
                                eprintln!("Error: could not disable {}: {}", dev_path, e);
                            }
                            std::process::exit(1);
                        }
                    }
                }
                println!("ZRAM devices disabled and reset");
            }
            #[cfg(not(target_os = "linux"))]
            {
                eprintln!("Error: ZRAM is only available on Linux");
                std::process::exit(1);
            }
        }
        "configure" => {
            if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
                print_subcommand_help("configure");
                return;
            }
            println!("ZRAM configuration not fully implemented in user-space.");
            println!("Use --help for more information.");
        }
        _ => {
            eprintln!("Error: unknown action '{}'", action);
            print_usage();
            std::process::exit(1);
        }
    }
}
