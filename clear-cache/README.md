# clear-cache

A multiplatform utility written in Rust to safely clear cache, temp files, and DNS cache to free up RAM and disk space, acting like a soft reboot without closing any active applications.

The goal is a "soft reboot": clear the accumulated dirt (cache, temp, logs, DNS,
page cache) so the machine feels freshly booted — without a real reboot and
without closing any running application.

## Features

- **Multiplatform**: Supports Linux, macOS, and Windows.
- **In-use detection & per-app confirmation**: On Linux, scans `/proc/<pid>/fd`
  to find which process currently holds each file open. Before deleting a file
  that is in use, it asks once per app — `[<PID>:<app>] está usando: <file> —
  Certeza que deseja apagar? [s/N]` — and applies your answer to every file held
  by that same process (grouped, so you are not asked file-by-file).
- **Error log cleanup**: Vacuums `systemd` journald, truncates active `*.log`
  files in `/var/log` (keeping writers' file handles valid), removes rotated
  logs (`*.log.1`, `*.gz`, `*.old`), and clears user logs
  (`~/.xsession-errors`, `~/.local/state`).
- **RAM/Page Cache release**: On Linux, syncs then instructs the kernel to drop
  page cache, dentries, and inodes (like a reboot).
- **DNS Flush**: Clears the system DNS cache resolver.
- **Dry-run**: Run with `--dry-run` to see what would be cleaned without
  modifying anything. In-use files are auto-skipped in dry-run (no prompts).
- **Non-interactive**: `-y`/`--yes` skips every in-use file without asking.
- **Statistics**: Shows total space freed and how many in-use files were kept.

## Usage

```
clear-cache [OPTIONS]

  -d, --dry-run   Trial run, delete nothing
  -v, --verbose   Show every file/dir touched
  -t, --temp      Clean temp dirs only
  -c, --cache     Clean cache dirs only
  -l, --logs      Clean error/system logs only
      --dns       Flush DNS resolver cache only
      --ram       Drop OS page caches only (Linux, needs root)
  -y, --yes       Non-interactive: skip all in-use files
  -h, --help      Help
```

With no category flags, everything runs. Journald, `/var/log`, and `--ram`
require running under `sudo`.
