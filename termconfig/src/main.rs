use std::env;
use std::mem;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    eprintln!("termconfig - change terminal settings");
    eprintln!();
    eprintln!("USAGE: termconfig [OPTIONS] [SETTING...]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -a          print all current settings");
    eprintln!("  -g          print settings in storable format");
    eprintln!("  -h, --help  display this help");
    eprintln!("  -v, --version display version");
    eprintln!();
    eprintln!("SETTINGS:");
    eprintln!("  raw         set raw mode");
    eprintln!("  cooked      set cooked mode");
    eprintln!("  echo        enable echoing");
    eprintln!("  -echo       disable echoing");
    eprintln!("  echoe       enable erasure");
    eprintln!("  echok       enable kill after newline");
    eprintln!("  sane        reset to sane defaults");
    eprintln!("  size        print terminal size");
}

fn get_termios() -> Option<libc::termios> {
    unsafe {
        let mut t: libc::termios = mem::zeroed();
        if libc::tcgetattr(libc::STDIN_FILENO, &mut t) == 0 {
            Some(t)
        } else {
            None
        }
    }
}

fn set_termios(t: &libc::termios) -> bool {
    unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, t) == 0 }
}

fn print_all_settings(t: &libc::termios) {
    println!("iflag:  {:06o}", t.c_iflag);
    println!("oflag:  {:06o}", t.c_oflag);
    println!("cflag:  {:06o}", t.c_cflag);
    println!("lflag:  {:06o}", t.c_lflag);
    println!("line:   {:02o}", t.c_line);
    println!("ispeed: {}", t.c_ispeed);
    println!("ospeed: {}", t.c_ospeed);

    let cc_names = [
        (0, "intr"),
        (1, "quit"),
        (2, "erase"),
        (3, "kill"),
        (4, "eof"),
        (5, "time"),
        (6, "min"),
        (7, "swtch"),
        (8, "start"),
        (9, "stop"),
        (10, "susp"),
        (11, "eol"),
        (12, "reprint"),
        (13, "discard"),
        (14, "werase"),
        (15, "lnext"),
    ];

    for (idx, name) in &cc_names {
        if *idx < t.c_cc.len() {
            let val = t.c_cc[*idx];
            if val < 128 {
                println!("cc[{}]: {:>10} = {:3} (^{})", idx, name, val, (val as u8 + b'@') as char);
            } else {
                println!("cc[{}]: {:>10} = {:3}", idx, name, val);
            }
        }
    }
}

fn print_storable(t: &libc::termios) {
    println!("{:04o}:{:06o}:{:06o}:{:06o}:{}", t.c_line, t.c_iflag, t.c_oflag, t.c_cflag, t.c_lflag);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut print_all = false;
    let mut print_storable_format = false;
    let mut settings: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("termconfig {}", VERSION);
                process::exit(0);
            }
            "-a" => print_all = true,
            "-g" => print_storable_format = true,
            _ if args[i].starts_with('-') => {
                eprintln!("termconfig: unknown option '{}'", args[i]);
                process::exit(2);
            }
            _ => {
                settings.push(args[i].clone());
            }
        }
        i += 1;
    }

    let mut t = match get_termios() {
        Some(t) => t,
        None => {
            eprintln!("termconfig: cannot get terminal settings");
            process::exit(1);
        }
    };

    if print_all {
        print_all_settings(&t);
        return;
    }

    if print_storable_format {
        print_storable(&t);
        return;
    }

    if settings.is_empty() {
        print_usage();
        process::exit(0);
    }

    for setting in &settings {
        match setting.as_str() {
            "raw" => {
                t.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
                t.c_iflag &= !(libc::IXON | libc::ICRNL | libc::BRKINT | libc::INPCK | libc::ISTRIP);
                t.c_oflag &= !libc::OPOST;
                t.c_cc[libc::VMIN] = 1;
                t.c_cc[libc::VTIME] = 0;
            }
            "cooked" => {
                t.c_lflag |= libc::ECHO | libc::ICANON | libc::ISIG;
                t.c_iflag |= libc::ICRNL;
                t.c_oflag |= libc::OPOST;
            }
            "echo" => {
                t.c_lflag |= libc::ECHO;
            }
            "-echo" => {
                t.c_lflag &= !libc::ECHO;
            }
            "echoe" => {
                t.c_lflag |= libc::ECHOE;
            }
            "echok" => {
                t.c_lflag |= libc::ECHOK;
            }
            "sane" => {
                t.c_lflag = libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN;
                t.c_iflag = libc::ICRNL | libc::IXON;
                t.c_oflag = libc::OPOST;
                t.c_cflag = libc::CS8 | libc::CREAD;
                t.c_cc[libc::VMIN] = 1;
                t.c_cc[libc::VTIME] = 0;
            }
            "size" => {
                unsafe {
                    let mut winsize: libc::winsize = mem::zeroed();
                    if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize) == 0 {
                        println!("{} {}", winsize.ws_row, winsize.ws_col);
                    } else {
                        eprintln!("termconfig: cannot get window size");
                    }
                }
                return;
            }
            _ => {
                eprintln!("termconfig: unknown setting '{}'", setting);
                process::exit(2);
            }
        }
    }

    if !set_termios(&t) {
        eprintln!("termconfig: cannot set terminal settings");
        process::exit(1);
    }
}
