use std::time::Instant;

fn print_usage() {
    eprintln!("Usage: {} [tamanho_mb] [iterações]", std::env::args().nth(0).unwrap_or_else(|| "zram-test".into()));
}

#[cfg(target_os = "linux")]
fn read_zram_stat(dev: &str, field: &str) -> Option<u64> {
    let path = format!("/sys/block/{}/{}", dev, field);
    std::fs::read_to_string(&path).ok().and_then(|s| s.trim().parse().ok())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            print_usage();
            println!("Benchmarks ZRAM compression performance.");
            println!("  <tamanho_mb>  Data size in MB (default: 50)");
            println!("  <iterações>   Number of iterations (default: 3)");
            println!("  --help, -h    Show this help message");
            println!("  --version     Show version information");
            return;
        }
        if arg == "--version" {
            println!("zram-test version 0.1.0");
            return;
        }
    }

    let size_mb: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(50);
    let iterations: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let size_bytes = size_mb * 1024 * 1024;

    println!("ZRAM Benchmark");
    println!("  Size: {} MB", size_mb);
    println!("  Iterations: {}", iterations);
    println!();

    #[cfg(target_os = "linux")]
    {
        let zram_devs: Vec<String> = match std::fs::read_dir("/sys/block") {
            Ok(dir) => dir
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with("zram"))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect(),
            Err(_) => vec![],
        };

        if zram_devs.is_empty() {
            println!("No ZRAM devices found. Run 'zram enable' first.");
            return;
        }

        for dev in &zram_devs {
            println!("Device: /dev/{}", dev);
            let _algo = read_zram_stat(dev, "comp_algorithm").unwrap_or(0);
            let disksize = read_zram_stat(dev, "disksize").unwrap_or(0);

            if disksize < size_bytes as u64 {
                println!("  Warning: device size ({} MB) may be smaller than test size", disksize / 1024 / 1024);
            }

            let mut total_write_time = 0.0f64;
            let mut total_read_time = 0.0f64;

            for iter in 1..=iterations {
                println!("  Iteration {}/{}:", iter, iterations);

                let data: Vec<u8> = (0..size_bytes).map(|i| (i % 256) as u8).collect();

                let write_start = Instant::now();
                std::fs::write(format!("/dev/{}", dev), &data).ok();
                let write_time = write_start.elapsed();
                total_write_time += write_time.as_secs_f64();

                let mut read_buf = vec![0u8; size_bytes];
                if let Ok(mut f) = std::fs::File::open(format!("/dev/{}", dev)) {
                    use std::io::Read;
                    let read_start = Instant::now();
                    f.read_exact(&mut read_buf).ok();
                    let read_time = read_start.elapsed();
                    total_read_time += read_time.as_secs_f64();
                }

                let orig_size = read_zram_stat(dev, "orig_data_size").unwrap_or(0);
                let comp_size = read_zram_stat(dev, "compr_data_size").unwrap_or(0);

                println!("    Write: {:.2}s", write_time.as_secs_f64());
                println!("    Read:  {:.2}s", (total_read_time / iter as f64));
                if comp_size > 0 {
                    let ratio = orig_size as f64 / comp_size as f64;
                    println!("    Ratio: {:.2}x", ratio);
                }
            }

            let avg_write = total_write_time / iterations as f64;
            let avg_read = total_read_time / iterations as f64;
            println!("  Average:");
            println!("    Write: {:.2}s ({:.2} MB/s)", avg_write, size_mb as f64 / avg_write);
            println!("    Read:  {:.2}s ({:.2} MB/s)", avg_read, size_mb as f64 / avg_read);
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("Error: ZRAM is only available on Linux");
        std::process::exit(1);
    }
}
