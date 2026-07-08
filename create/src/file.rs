use std::fs;
use std::io::ErrorKind;

pub(crate) fn describe_error_kind(kind: ErrorKind, already_exists: &'static str) -> &'static str {
    match kind {
        ErrorKind::NotFound => "File not found",
        ErrorKind::PermissionDenied => "Do not have permissions",
        ErrorKind::ConnectionRefused => "Connection refused",
        ErrorKind::ConnectionReset => "Connection reset",
        ErrorKind::HostUnreachable => "Host unreachable",
        ErrorKind::NetworkUnreachable => "Network unreachable",
        ErrorKind::ConnectionAborted => "Connection aborted",
        ErrorKind::NotConnected => "Not connected",
        ErrorKind::AddrInUse => "Address in use",
        ErrorKind::AddrNotAvailable => "Address not available",
        ErrorKind::NetworkDown => "Network down",
        ErrorKind::BrokenPipe => "Broken pipe",
        ErrorKind::AlreadyExists => already_exists,
        ErrorKind::WouldBlock => "Operation would block",
        ErrorKind::NotADirectory => "Path is not a directory",
        ErrorKind::IsADirectory => "Path is a directory",
        ErrorKind::DirectoryNotEmpty => "Directory not empty",
        ErrorKind::ReadOnlyFilesystem => "Filesystem is read-only",
        ErrorKind::FilesystemLoop => "Filesystem loop detected",
        ErrorKind::StaleNetworkFileHandle => "Stale network file handle",
        ErrorKind::InvalidInput => "Invalid input",
        ErrorKind::InvalidData => "Invalid data",
        ErrorKind::TimedOut => "Operation timed out",
        ErrorKind::WriteZero => "Write returned zero",
        ErrorKind::StorageFull => "Storage full",
        ErrorKind::NotSeekable => "Not seekable",
        ErrorKind::QuotaExceeded => "Quota exceeded",
        ErrorKind::FileTooLarge => "File too large",
        ErrorKind::ResourceBusy => "Resource busy",
        ErrorKind::ExecutableFileBusy => "Executable file busy",
        ErrorKind::Deadlock => "Deadlock detected",
        ErrorKind::CrossesDevices => "Operation crosses devices",
        ErrorKind::TooManyLinks => "Too many links",
        ErrorKind::InvalidFilename => "Invalid filename",
        ErrorKind::ArgumentListTooLong => "Argument list too long",
        ErrorKind::Interrupted => "Operation interrupted",
        ErrorKind::Unsupported => "Operation not supported",
        ErrorKind::UnexpectedEof => "Unexpected end of file",
        ErrorKind::OutOfMemory => "Out of memory",
        ErrorKind::InProgress => "Operation in progress",
        ErrorKind::TooManyOpenFiles => "Too many open files",
        ErrorKind::Other => "Other error occurred",
        _ => "Unknown error occurred",
    }
}

fn write_file(path: &str, content: &str, recursive: bool) {
    if recursive {
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(e) = fs::create_dir_all(parent) {
                    println!("{}", describe_error_kind(e.kind(), "Directory already exists"));
                    std::process::exit(1);
                }
            }
        }
    }

    match fs::write(path, content) {
        Ok(_) => println!("File {} was created with success", path),
        Err(e) => {
            println!("{}", describe_error_kind(e.kind(), "File already exists"));
            std::process::exit(1);
        }
    }
}

pub fn create(path: &str, recursive: bool) {
    if path.is_empty() {
        panic!("use: create -f <path>\n'path' arg is required")
    }

    write_file(path, "", recursive);
}

pub fn create_with_content(path: &str, content: &str, recursive: bool) {
    if path.is_empty() {
        panic!("use: create -f <path>\n'path' arg is required")
    }

    if content.is_empty() {
        panic!("us: create -f <path> -c <content>\n'content' arg is required")
    }

    write_file(path, content, recursive);
}
