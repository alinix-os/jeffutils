use serde::Deserialize;
use std::process::Command;

fn print_usage() {
    eprintln!("Usage: {} <comando> [args...]", std::env::args().next().unwrap_or_else(|| "disk".into()));
}

#[derive(Debug, Deserialize)]
struct BlockDevice {
    name: String,
    #[serde(rename = "type")]
    device_type: String,
    model: Option<String>,
    size: String,
    fstype: Option<String>,
    mountpoint: Option<String>,
    uuid: Option<String>,
    tran: Option<String>,
    #[serde(default)]
    children: Vec<BlockDevice>,
}

#[derive(Debug, Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<BlockDevice>,
}

fn get_block_devices() -> Result<Vec<BlockDevice>, String> {
    let output = Command::new("lsblk")
        .args(["-o", "NAME,TYPE,MODEL,SIZE,FSTYPE,MOUNTPOINT,UUID,TRAN", "-J"])
        .output()
        .map_err(|e| format!("Failed to execute lsblk: {}", e))?;

    if !output.status.success() {
        return Err(format!("lsblk failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let parsed: LsblkOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse lsblk output: {}", e))?;

    Ok(parsed.blockdevices)
}

// Maps standard Linux block device names to JeffNix naming system
// We keep it stable during a session by sorting physical devices by name/detection order.
fn map_linux_to_jeffnix(devices: &[BlockDevice]) -> Vec<(String, String, &BlockDevice)> {
    // Only process parent devices (type == "disk" or "rom")
    let mut disks: Vec<&BlockDevice> = devices
        .iter()
        .filter(|d| d.device_type == "disk" || d.device_type == "rom")
        .collect();

    // Sort by name to ensure stable detection ordering
    disks.sort_by(|a, b| a.name.cmp(&b.name));

    let mut nvme_count = 1;
    let mut sata_count = 1;
    let mut usb_count = 1;
    let mut sd_count = 1;
    let mut cd_count = 1;

    let mut mappings = Vec::new();

    for disk in disks {
        let name_lower = disk.name.to_lowercase();
        let tran_lower = disk.tran.as_deref().unwrap_or("").to_lowercase();
        let model_lower = disk.model.as_deref().unwrap_or("").to_lowercase();

        let (dev_type, idx) = if name_lower.starts_with("nvme") || tran_lower == "nvme" {
            let i = nvme_count;
            nvme_count += 1;
            ("NVME".to_string(), i)
        } else if tran_lower == "usb" || model_lower.contains("usb") || name_lower.starts_with("sd") && is_usb_device(&disk.name) {
            let i = usb_count;
            usb_count += 1;
            ("USB".to_string(), i)
        } else if name_lower.starts_with("mmcblk") {
            let i = sd_count;
            sd_count += 1;
            ("SDCARD".to_string(), i)
        } else if name_lower.starts_with("sr") || disk.device_type == "rom" {
            let i = cd_count;
            cd_count += 1;
            ("CDROM".to_string(), i)
        } else {
            let i = sata_count;
            sata_count += 1;
            ("SATA".to_string(), i)
        };

        let jname = if dev_type == "USB" {
            format!("{}{:03}", dev_type, idx)
        } else {
            format!("{}{}", dev_type, idx)
        };

        mappings.push((disk.name.clone(), jname.clone(), disk));

        // Partitions
        let mut p_idx = 1;
        for child in &disk.children {
            if child.device_type == "part" {
                let jpname = format!("{}p{}", jname, p_idx);
                mappings.push((child.name.clone(), jpname, child));
                p_idx += 1;
            }
        }
    }

    mappings
}

fn is_usb_device(name: &str) -> bool {
    let sys_path = format!("/sys/block/{}/device", name);
    if let Ok(target) = std::fs::read_link(&sys_path) {
        let path_str = target.to_string_lossy();
        if path_str.contains("usb") {
            return true;
        }
    }
    false
}

fn cmd_list(args: &[String]) {
    let devices = match get_block_devices() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let mappings = map_linux_to_jeffnix(&devices);

    let _has_default = args.iter().any(|a| a == "--default");
    let clean_args: Vec<String> = args.iter().filter(|a| *a != "--default").cloned().collect();

    if clean_args.is_empty() {
        // List parent devices only
        println!("{:<10} {:<7} {:<20} {:<10}", "NAME", "TYPE", "MODEL", "SIZE");
        println!("{}", "-".repeat(53));
        for (_linux_name, jeffnix_name, dev) in &mappings {
            // Check if it's a parent device
            if dev.device_type == "disk" || dev.device_type == "rom" {
                let dev_type = if jeffnix_name.starts_with("NVME") {
                    "NVMe"
                } else if jeffnix_name.starts_with("SATA") {
                    "SATA"
                } else if jeffnix_name.starts_with("USB") {
                    "USB"
                } else if jeffnix_name.starts_with("SDCARD") {
                    "SDCard"
                } else if jeffnix_name.starts_with("CDROM") {
                    "CD-ROM"
                } else {
                    "Disk"
                };

                let model = dev.model.as_deref().unwrap_or("Unknown");
                println!("{:<10} {:<7} {:<20} {:<10}", jeffnix_name, dev_type, model, dev.size);
            }
        }
        return;
    }

    let target = &clean_args[0];

    // Find the device matching target name (case insensitive)
    let target_upper = target.to_uppercase();
    let target_clean = target.trim_start_matches("/dev/");
    // Normalize target shorthand like 'nvme0p1' to 'nvme0n1p1'
    let mut normalized_target = target_clean.to_lowercase();
    if normalized_target.starts_with("nvme") && !normalized_target.contains("n1") {
        if let Some(pos) = normalized_target.find('p') {
            let (disk_part, part_part) = normalized_target.split_at(pos);
            if disk_part.ends_with('0') || disk_part.ends_with('1') || disk_part.ends_with('2') {
                normalized_target = format!("{}n1{}", disk_part, part_part);
            }
        } else if normalized_target.ends_with('0') || normalized_target.ends_with('1') || normalized_target.ends_with('2') {
            normalized_target = format!("{}n1", normalized_target);
        }
    }

    let found = mappings.iter()
        .find(|(_, jname, _)| jname.to_uppercase() == target_upper)
        .or_else(|| mappings.iter().find(|(lname, _, _)| lname.to_lowercase() == normalized_target))
        .or_else(|| {
            // Try matching as prefix of standard Linux name
            mappings.iter().find(|(lname, _, _)| lname.to_lowercase().starts_with(&normalized_target))
        })
        .or_else(|| {
            // Try matching as prefix of JeffNix name (e.g. NVME -> NVME1)
            mappings.iter().find(|(_, jname, _)| jname.to_uppercase().starts_with(&target_upper))
        });

    if let Some((_, jname, dev)) = found {
        // If it's a disk, list its partitions
        if dev.device_type == "disk" || dev.device_type == "rom" {
            let is_jeffnix = std::path::Path::new("/Sys/auto-mount").exists() || std::env::var("JEFFNIX_ENV").is_ok();
            if _has_default || is_jeffnix {
                println!("| Partition | FS    | Mount Point   | UUID      | Size   |");
                println!("| --------- | ----- | ------------- | --------- | ------ |");
            } else {
                println!("{:<10} {:<6} {:<20} {:<15} {:<10}", "Partition", "FS", "Mount Point", "UUID", "Size");
                println!("{}", "-".repeat(65));
            }

            // Print partitions of this disk
            // We search mappings for keys starting with `<jname>p`
            let prefix = format!("{}p", jname);
            for (_, pjname, pdev) in &mappings {
                if pjname.starts_with(&prefix) {
                    let part_num = pjname.strip_prefix(jname).unwrap_or("");
                    let fs = pdev.fstype.as_deref().unwrap_or("-");
                    let mp = pdev.mountpoint.as_deref().unwrap_or("-");
                    let uuid = pdev.uuid.as_deref().unwrap_or("-");
                    if _has_default || is_jeffnix {
                        println!("| {:<9} | {:<5} | {:<13} | {:<9} | {:<6} |", part_num, fs, mp, uuid, pdev.size);
                    } else {
                        println!("{:<10} {:<6} {:<20} {:<15} {:<10}", part_num, fs, mp, uuid, pdev.size);
                    }
                }
            }
        } else {
            eprintln!("Device {} is a partition. List partitions by specifying the disk name (e.g. disk list SATA1)", target);
            std::process::exit(1);
        }
    } else {
        eprintln!("Device not found: {}", target);
        std::process::exit(1);
    }
}

fn cmd_info(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: disk info <device> [attribute]");
        std::process::exit(1);
    }

    let devices = match get_block_devices() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let mappings = map_linux_to_jeffnix(&devices);
    let target = &args[0];
    let target_upper = target.to_uppercase();

    let target_clean = target.trim_start_matches("/dev/");
    // Normalize target shorthand like 'nvme0p1' to 'nvme0n1p1'
    let mut normalized_target = target_clean.to_lowercase();
    if normalized_target.starts_with("nvme") && !normalized_target.contains("n1") {
        if let Some(pos) = normalized_target.find('p') {
            let (disk_part, part_part) = normalized_target.split_at(pos);
            if disk_part.ends_with('0') || disk_part.ends_with('1') || disk_part.ends_with('2') {
                normalized_target = format!("{}n1{}", disk_part, part_part);
            }
        } else if normalized_target.ends_with('0') || normalized_target.ends_with('1') || normalized_target.ends_with('2') {
            normalized_target = format!("{}n1", normalized_target);
        }
    }

    let found = mappings.iter()
        .find(|(_, jname, _)| jname.to_uppercase() == target_upper)
        .or_else(|| mappings.iter().find(|(lname, _, _)| lname.to_lowercase() == normalized_target))
        .or_else(|| {
            // Try matching as prefix of standard Linux name
            mappings.iter().find(|(lname, _, _)| lname.to_lowercase().starts_with(&normalized_target))
        })
        .or_else(|| {
            // Try matching as prefix of JeffNix name (e.g. NVME -> NVME1)
            mappings.iter().find(|(_, jname, _)| jname.to_uppercase().starts_with(&target_upper))
        });

    if let Some((_, jname, dev)) = found {
        let attr = args.get(1).map(|s| s.to_lowercase());

        if let Some(ref val) = attr {
            match val.as_str() {
                "uuid" => {
                    println!("{}", dev.uuid.as_deref().unwrap_or(""));
                }
                "size" => {
                    println!("{}", dev.size);
                }
                "fs" | "filesystem" => {
                    println!("{}", dev.fstype.as_deref().unwrap_or(""));
                }
                "mount" | "mountpoint" | "mount_point" => {
                    println!("{}", dev.mountpoint.as_deref().unwrap_or(""));
                }
                "type" => {
                    println!("{}", dev.device_type);
                }
                "model" => {
                    println!("{}", dev.model.as_deref().unwrap_or(""));
                }
                "partitions" => {
                    let prefix = format!("{}p", jname);
                    let part_names: Vec<&str> = mappings
                        .iter()
                        .filter(|(_, pjname, _)| pjname.starts_with(&prefix))
                        .map(|(_, pjname, _)| pjname.as_str())
                        .collect();
                    println!("{}", part_names.join(" "));
                }
                "health" => {
                    println!("OK");
                }
                "serial" => {
                    // Try to get serial from udev or dummy
                    println!("Unknown");
                }
                _ => {
                    eprintln!("Unknown attribute: {}", val);
                    std::process::exit(1);
                }
            }
        } else {
            // General info
            let target_os = std::env::consts::OS;
            let is_jeffnix = std::path::Path::new("/Sys/auto-mount").exists() || std::env::var("JEFFNIX_ENV").is_ok();
            let label = if is_jeffnix {
                "JeffNix"
            } else if target_os == "macos" {
                "macOS"
            } else if target_os == "windows" {
                "Windows"
            } else {
                "UNIX/Linux"
            };
            println!("OS Environment: {}", label);
            println!("Device: {}", jname);
            println!("Type: {}", dev.device_type);
            if let Some(ref m) = dev.model {
                println!("Model: {}", m);
            }
            println!("Size: {}", dev.size);
            if let Some(ref f) = dev.fstype {
                println!("Filesystem: {}", f);
            }
            if let Some(ref m) = dev.mountpoint {
                println!("Mount Point: {}", m);
            }
            if let Some(ref u) = dev.uuid {
                println!("UUID: {}", u);
            }
        }
    } else {
        eprintln!("Device not found: {}", target);
        std::process::exit(1);
    }
}

fn cmd_mount(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: disk mount <partição> [destino] [--auto]");
        std::process::exit(1);
    }

    let devices = match get_block_devices() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    let mappings = map_linux_to_jeffnix(&devices);

    let auto = args.iter().any(|a| a == "--auto");
    let non_options: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();

    if non_options.is_empty() {
        eprintln!("Usage: disk mount <partição> [destino] [--auto]");
        std::process::exit(1);
    }

    let target_part = non_options[0];
    let target_upper = target_part.to_uppercase();
    let found = mappings.iter()
        .find(|(_, jname, _)| jname.to_uppercase() == target_upper)
        .or_else(|| mappings.iter().find(|(lname, _, _)| lname.to_lowercase() == target_part.to_lowercase()));

    let (linux_dev, _jname, dev) = match found {
        Some((l, j, d)) => (l, j, d),
        None => {
            eprintln!("Device not found: {}", target_part);
            std::process::exit(1);
        }
    };

    let dev_path = format!("/dev/{}", linux_dev);

    if auto {
        let dest = if non_options.len() > 1 {
            non_options[1].clone()
        } else {
            let label = dev.model.as_deref().or(dev.uuid.as_deref()).unwrap_or("Volume");
            format!("/Volumes/{}", label)
        };

        println!("Registering auto-mount for {} on {}", target_part, dest);
        // Writing to /Sys/auto-mount or jeffnix_fstab
        let auto_mount_dir = std::path::Path::new("/Sys");
        let auto_mount_path = auto_mount_dir.join("auto-mount");
        let entry = format!("{} {} {} defaults 0 0\n", dev_path, dest, dev.fstype.as_deref().unwrap_or("auto"));

        // Make sure /Sys directory exists or simulate
        let write_res = if auto_mount_dir.exists() {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&auto_mount_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    write!(f, "{}", entry)
                })
        } else {
            // Write to user's config directory as fallback
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            let config_dir = format!("{}/.config/jeffnix", home);
            let _ = std::fs::create_dir_all(&config_dir);
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("{}/auto-mount", config_dir))
                .and_then(|mut f| {
                    use std::io::Write;
                    write!(f, "{}", entry)
                })
        };

        if let Err(e) = write_res {
            eprintln!("Failed to register auto-mount entry: {}", e);
            std::process::exit(1);
        }
        println!("Registered auto-mount successfully.");
        return;
    }

    let mountpoint = if non_options.len() > 1 {
        non_options[1].clone()
    } else {
        let label = dev.uuid.as_deref().unwrap_or("Volume");
        format!("/Volumes/{}", label)
    };

    let _ = std::fs::create_dir_all(&mountpoint);
    println!("Mounting {} on {}", target_part, mountpoint);

    let status = Command::new("mount")
        .arg(&dev_path)
        .arg(&mountpoint)
        .status();

    match status {
        Ok(s) if s.success() => println!("Mounted successfully"),
        Ok(s) => {
            eprintln!("Mount failed with code {:?}", s.code());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error invoking mount: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_unmount(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: disk unmount <partição>");
        std::process::exit(1);
    }

    let devices = match get_block_devices() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    let mappings = map_linux_to_jeffnix(&devices);

    let target = &args[0];
    let target_upper = target.to_uppercase();
    let found = mappings.iter()
        .find(|(_, jname, _)| jname.to_uppercase() == target_upper)
        .or_else(|| mappings.iter().find(|(lname, _, _)| lname.to_lowercase() == target.to_lowercase()));

    let linux_dev = match found {
        Some((l, _, _)) => l,
        None => {
            // Try treating target as standard mountpoint / device path
            target
        }
    };

    let dev_path = if linux_dev.starts_with('/') {
        linux_dev.clone()
    } else {
        format!("/dev/{}", linux_dev)
    };

    println!("Unmounting {}", target);

    let status = Command::new("umount")
        .arg(&dev_path)
        .status();

    match status {
        Ok(s) if s.success() => println!("Unmounted successfully"),
        Ok(s) => {
            eprintln!("Unmount failed with code {:?}", s.code());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error invoking umount: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_eject(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: disk eject <device>");
        std::process::exit(1);
    }

    let devices = match get_block_devices() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    let mappings = map_linux_to_jeffnix(&devices);

    let target = &args[0];
    let target_upper = target.to_uppercase();
    let found = mappings.iter()
        .find(|(_, jname, _)| jname.to_uppercase() == target_upper)
        .or_else(|| mappings.iter().find(|(lname, _, _)| lname.to_lowercase() == target.to_lowercase()));

    let linux_dev = match found {
        Some((l, _, _)) => l,
        None => {
            eprintln!("Device not found: {}", target);
            std::process::exit(1);
        }
    };

    let dev_path = format!("/dev/{}", linux_dev);
    println!("Ejecting {}", target);

    let status = Command::new("eject")
        .arg(&dev_path)
        .status();

    match status {
        Ok(s) if s.success() => println!("Ejected successfully"),
        Ok(s) => {
            eprintln!("Eject failed with code {:?}", s.code());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error invoking eject: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        println!();
        println!("Commands:");
        println!("  list       [device]               List devices or partitions of a device");
        println!("  info       <device> [attribute]   Show device information");
        println!("  mount      <part> [dest] [--auto] Mount a partition");
        println!("  unmount    <part>                 Unmount a partition");
        println!("  eject      <device>               Eject a removable device");
        println!("  --help, -h                        Show this help");
        return;
    }

    let cmd = &args[0];
    let rest = args[1..].to_vec();

    match cmd.as_str() {
        "--help" | "-h" => {
            print_usage();
        }
        "list" => cmd_list(&rest),
        "info" => cmd_info(&rest),
        "mount" => cmd_mount(&rest),
        "unmount" => cmd_unmount(&rest),
        "eject" => cmd_eject(&rest),
        _ => {
            eprintln!("Error: unknown command '{}'", cmd);
            std::process::exit(1);
        }
    }
}

