use std::env;

const VERSION: &str = "0.1.0";

fn print_usage() {
    let name = env::args().next().unwrap_or_else(|| "online".into());
    eprintln!("Usage: {name} [-b]");
    eprintln!("Show who is logged in.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -b          brief: print only username and terminal");
    eprintln!("  -h, --help  show this help message");
    eprintln!("  -v, --version show version");
}

fn format_time(tv_sec: i64) -> String {
    if tv_sec == 0 {
        return String::new();
    }
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&tv_sec, &mut tm);
        let mut buf = [0u8; 64];
        let fmt = b"%Y-%m-%d %H:%M\0";
        let len = libc::strftime(
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            fmt.as_ptr() as *const libc::c_char,
            &tm,
        );
        if len > 0 {
            String::from_utf8_lossy(&buf[..len]).into_owned()
        } else {
            format!("{tv_sec}")
        }
    }
}

fn format_idle(tv_sec: i64) -> String {
    if tv_sec == 0 {
        return "  .  ".to_string();
    }
    let days = tv_sec / 86400;
    let hours = (tv_sec % 86400) / 3600;
    let mins = (tv_sec % 3600) / 60;
    if days > 0 {
        format!("{days:>2}:{hours:02}:{mins:02}")
    } else {
        format!("{hours:>2}:{mins:02}")
    }
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("online", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_usage();
            return;
        }
        if arg == "-v" || arg == "--version" {
            println!("online {VERSION}");
            return;
        }
    }

    let brief = args.iter().any(|a| a == "-b");

    let now = unsafe { libc::time(std::ptr::null_mut()) };

    unsafe {
        libc::setutxent();

        loop {
            let entry = libc::getutxent();
            if entry.is_null() {
                break;
            }
            let entry = &*entry;

            if entry.ut_type != libc::USER_PROCESS {
                continue;
            }

            let username = std::ffi::CStr::from_ptr(entry.ut_user.as_ptr())
                .to_string_lossy()
                .into_owned();
            if username.is_empty() || username == "LOGIN" {
                continue;
            }

            let terminal = std::ffi::CStr::from_ptr(entry.ut_line.as_ptr())
                .to_string_lossy()
                .into_owned();

            let pid = entry.ut_pid;

            let tv_sec = entry.ut_tv.tv_sec as i64;
            let login_time = format_time(tv_sec);

            if brief {
                println!("{username:<12} {terminal}");
            } else {
                let idle_secs = now - tv_sec;
                let idle = format_idle(if idle_secs > 0 { idle_secs } else { 0 });
                let host = std::ffi::CStr::from_ptr(entry.ut_host.as_ptr())
                    .to_string_lossy()
                    .into_owned();
                let from = if host.is_empty() {
                    String::new()
                } else {
                    format!(" ({host})")
                };
                println!(
                    "{username:<12} {terminal:<10} {login_time:<16} {idle}  {pid}{from}"
                );
            }
        }

        libc::endutxent();
    }
}
