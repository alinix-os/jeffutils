use std::time::Instant;

fn print_usage() {
    eprintln!("Usage: {} [tamanho_mb]", std::env::args().nth(0).unwrap_or_else(|| "mem-test".into()));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                println!("Runs a memory test by allocating and writing/reading patterns.");
                println!("  <tamanho_mb>  Size to test in MB (default: 100)");
                println!("  --help, -h    Show this help message");
                println!("  --version     Show version information");
                return;
            }
            "--version" => {
                println!("mem-test version 0.1.0");
                return;
            }
            _ => {}
        }
    }

    let size_mb: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(100);
    let size_bytes = size_mb * 1024 * 1024;

    println!("Memory Test: {} MB", size_mb);
    println!("Allocating...");
    let start = Instant::now();

    let mut vec: Vec<u8> = Vec::with_capacity(size_bytes);
    vec.resize(size_bytes, 0);
    let alloc_time = start.elapsed();

    println!("  Allocation: {:?}", alloc_time);

    println!("Writing pattern (0xAA)...");
    let write_start = Instant::now();
    vec.fill(0xAA);
    let write_time = write_start.elapsed();
    println!("  Write: {:?} ({:.2} MB/s)", write_time, size_mb as f64 / write_time.as_secs_f64());

    println!("Reading and verifying...");
    let read_start = Instant::now();
    if !vec.iter().all(|&b| b == 0xAA) {
        eprintln!("  Error: mismatch in 0xAA pattern");
        std::process::exit(1);
    }
    let read_time = read_start.elapsed();
    println!("  Read & Verify: {:?} ({:.2} MB/s)", read_time, size_mb as f64 / read_time.as_secs_f64());

    println!("Writing pattern (0x55)...");
    vec.fill(0x55);

    println!("Verifying 0x55 pattern...");
    if !vec.iter().all(|&b| b == 0x55) {
        eprintln!("  Error: mismatch in 0x55 pattern");
        std::process::exit(1);
    }

    println!("Deallocating...");
    drop(vec);
    println!("Test complete: {} MB passed all checks", size_mb);
    println!("Total time: {:?}", start.elapsed());
}
