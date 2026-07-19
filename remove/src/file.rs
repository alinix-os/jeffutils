use std::fs;
use std::io::ErrorKind;

pub(crate) fn describe_error_kind(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "File not found",
        ErrorKind::PermissionDenied => "Do not have permissions",
        ErrorKind::ConnectionRefused => "Connection refused",
        ErrorKind::ConnectionReset => "Connection reset",
        ErrorKind::ConnectionAborted => "Connection aborted",
        ErrorKind::NotConnected => "Not connected",
        ErrorKind::AddrInUse => "Address in use",
        ErrorKind::AddrNotAvailable => "Address not available",
        ErrorKind::BrokenPipe => "Broken pipe",
        ErrorKind::AlreadyExists => "File already exists",
        ErrorKind::WouldBlock => "Operation would block",
        ErrorKind::InvalidInput => "Invalid input",
        ErrorKind::InvalidData => "Invalid data",
        ErrorKind::TimedOut => "Operation timed out",
        ErrorKind::WriteZero => "Write returned zero",
        ErrorKind::Interrupted => "Operation interrupted",
        ErrorKind::Unsupported => "Operation not supported",
        ErrorKind::UnexpectedEof => "Unexpected end of file",
        ErrorKind::OutOfMemory => "Out of memory",
        ErrorKind::Other => "Other error occurred",
        _ => "Unknown error occurred",
    }
}

pub fn remove(path: &str) {
    if path.is_empty() {
        eprintln!("use: remove <path> -f\n'path' arg is required");
        std::process::exit(1);
    }

    match fs::remove_file(path) {
        Ok(_) => println!("File {} was removed with success", path),
        Err(e) => {
            eprintln!("{}", describe_error_kind(e.kind()));
            std::process::exit(1);
        }
    }
}
