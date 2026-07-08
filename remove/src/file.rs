use std::fs;
use std::io::ErrorKind;

pub(crate) fn describe_error_kind(kind: ErrorKind) -> &'static str {
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
        ErrorKind::AlreadyExists => "File already exists",
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

pub fn remove(path: &str) {
    if path.is_empty() {
        panic!("use: remove <path> -f\n'path' arg is required")
    }

    match fs::remove_file(path) {
        Ok(_) => println!("File {} was removed with success", path),
        Err(e) => {
            println!("{}", describe_error_kind(e.kind()));
            std::process::exit(1);
        }
    }
}
