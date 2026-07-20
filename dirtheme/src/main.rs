use std::env;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("dirtheme - output shell code for LS_COLORS");
    eprintln!();
    eprintln!("USAGE: dirtheme [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -p          output default configuration");
    eprintln!("  -b          Bourne shell format (default)");
    eprintln!("  -c          C shell format");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
}

fn default_ls_colors() -> Vec<(&'static str, &'static str)> {
    vec![
        ("di", "01;34"),
        ("ln", "01;36"),
        ("so", "01;35"),
        ("pi", "33"),
        ("bd", "01;33"),
        ("cd", "01;33"),
        ("or", "40;31;01"),
        ("mi", "40;31;01"),
        ("ex", "01;32"),
        ("*.tar", "01;31"),
        ("*.tgz", "01;31"),
        ("*.arc", "01;31"),
        ("*.arj", "01;31"),
        ("*.taz", "01;31"),
        ("*.lha", "01;31"),
        ("*.lz4", "01;31"),
        ("*.lzh", "01;31"),
        ("*.lzma", "01;31"),
        ("*.tlz", "01;31"),
        ("*.tzo", "01;31"),
        ("*.t7z", "01;31"),
        ("*.zip", "01;31"),
        ("*.z", "01;31"),
        ("*.dz", "01;31"),
        ("*.gz", "01;31"),
        ("*.lz", "01;31"),
        ("*.lzo", "01;31"),
        ("*.xz", "01;31"),
        ("*.zst", "01;31"),
        ("*.tzst", "01;31"),
        ("*.bz2", "01;31"),
        ("*.bz", "01;31"),
        ("*.tbz", "01;31"),
        ("*.tbz2", "01;31"),
        ("*.tz", "01;31"),
        ("*.deb", "01;31"),
        ("*.rpm", "01;31"),
        ("*.jar", "01;31"),
        ("*.war", "01;31"),
        ("*.ear", "01;31"),
        ("*.sar", "01;31"),
        ("*.rar", "01;31"),
        ("*.ace", "01;31"),
        ("*.zoo", "01;31"),
        ("*.cpio", "01;31"),
        ("*.7z", "01;31"),
        ("*.rz", "01;31"),
        ("*.cab", "01;31"),
        ("*.jpg", "01;35"),
        ("*.jpeg", "01;35"),
        ("*.mjpg", "01;35"),
        ("*.mjpeg", "01;35"),
        ("*.gif", "01;35"),
        ("*.bmp", "01;35"),
        ("*.pbm", "01;35"),
        ("*.pgm", "01;35"),
        ("*.ppm", "01;35"),
        ("*.tga", "01;35"),
        ("*.xbm", "01;35"),
        ("*.xpm", "01;35"),
        ("*.tif", "01;35"),
        ("*.tiff", "01;35"),
        ("*.png", "01;35"),
        ("*.svg", "01;35"),
        ("*.svgz", "01;35"),
        ("*.webp", "01;35"),
        ("*.ico", "01;35"),
        ("*.mov", "01;35"),
        ("*.mpg", "01;35"),
        ("*.mpeg", "01;35"),
        ("*.m2v", "01;35"),
        ("*.mkv", "01;35"),
        ("*.webm", "01;35"),
        ("*.ogm", "01;35"),
        ("*.mp4", "01;35"),
        ("*.m4v", "01;35"),
        ("*.mp4v", "01;35"),
        ("*.vob", "01;35"),
        ("*.ogv", "01;35"),
        ("*.avi", "01;35"),
        ("*.wmv", "01;35"),
        ("*.f4v", "01;35"),
        ("*.flv", "01;35"),
        ("*.aac", "00;36"),
        ("*.au", "00;36"),
        ("*.flac", "00;36"),
        ("*.m4a", "00;36"),
        ("*.mid", "00;36"),
        ("*.midi", "00;36"),
        ("*.mka", "00;36"),
        ("*.mp3", "00;36"),
        ("*.mpc", "00;36"),
        ("*.ogg", "00;36"),
        ("*.opus", "00;36"),
        ("*.ra", "00;36"),
        ("*.wav", "00;36"),
        ("*.oga", "00;36"),
        ("*.opus", "00;36"),
        ("*.spx", "00;36"),
        ("*.xspf", "00;36"),
        ("*.py", "00;32"),
        ("*.pl", "00;32"),
        ("*.pm", "00;32"),
        ("*.rb", "00;32"),
        ("*.sh", "00;32"),
        ("*.bash", "00;32"),
        ("*.csh", "00;32"),
        ("*.zsh", "00;32"),
        ("*.c", "00;37"),
        ("*.h", "00;37"),
        ("*.cpp", "00;37"),
        ("*.hpp", "00;37"),
        ("*.rs", "00;37"),
        ("*.go", "00;37"),
        ("*.java", "00;37"),
        ("*.js", "00;37"),
        ("*.ts", "00;37"),
        ("*.jsx", "00;37"),
        ("*.tsx", "00;37"),
        ("*.vue", "00;37"),
        ("*.css", "00;37"),
        ("*.scss", "00;37"),
        ("*.html", "00;37"),
        ("*.htm", "00;37"),
        ("*.xml", "00;37"),
        ("*.json", "00;37"),
        ("*.yaml", "00;37"),
        ("*.yml", "00;37"),
        ("*.toml", "00;37"),
        ("*.md", "00;37"),
        ("*.txt", "00;37"),
        ("*.csv", "00;37"),
        ("*.log", "00;37"),
        ("*.conf", "00;37"),
        ("*.cfg", "00;37"),
        ("*.ini", "00;37"),
        ("*.sql", "00;37"),
        ("*.diff", "00;37"),
        ("*.patch", "00;37"),
    ]
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("dirtheme", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    let mut print_default = false;
    let mut c_format = false;

    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("dirtheme {}", VERSION);
                process::exit(0);
            }
            "-p" => print_default = true,
            "-b" => c_format = false,
            "-c" => c_format = true,
            _ => {
                eprintln!("dirtheme: unknown option '{}'", arg);
                process::exit(2);
            }
        }
    }

    if !print_default {
        print_usage();
        process::exit(0);
    }

    let entries = default_ls_colors();

    if c_format {
        println!("setenv LS_COLORS '");
        for (pattern, color) in &entries {
            println!("{}={}", pattern, color);
        }
        print!(":'");
        println!();
    } else {
        print!("LS_COLORS='");
        for (i, (pattern, color)) in entries.iter().enumerate() {
            if i > 0 {
                print!(":");
            }
            print!("{}={}", pattern, color);
        }
        println!("';");
        println!("export LS_COLORS");
    }
}
