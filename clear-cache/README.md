# clear-cache

A multiplatform utility written in Rust to safely clear cache, temp files, and DNS cache to free up RAM and disk space, acting like a soft reboot without closing any active applications.

## Features

- **Multiplatform**: Supports Linux, macOS, and Windows.
- **Safe**: Files that are currently locked or in use by active applications are automatically skipped, ensuring no running applications crash or close.
- **RAM/Page Cache release**: On Linux, instructs the kernel to drop page cache, dentries, and inodes (like a reboot).
- **DNS Flush**: Clears the system DNS cache resolver.
- **Dry-run**: Run with `--dry-run` to see what would be cleaned without modifying anything.
- **Statistics**: Shows total disk space freed.
