// fileio module — reading from and writing to files on disk

use std::fs;
use std::io;
use std::path::Path;

/// Reads a file and returns its lines as a vector of Strings
///
/// * `path` — path to the file to read
///
/// Returns `Ok(Vec<String>)` with each line of the file,
/// or `Err(io::Error)` if reading fails.
pub fn read_file(path: &Path) -> io::Result<Vec<String>> {
    let content = fs::read_to_string(path)?;

    // Split content into lines, preserving empty lines
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    Ok(lines)
}

/// Saves the text buffer to a file
///
/// * `path` — path where to save the file
/// * `lines` — slice of lines from the buffer
///
/// Returns `Ok(())` on success, or `Err(io::Error)` on failure.
pub fn save_file(path: &Path, lines: &[String]) -> io::Result<()> {
    // Concatenate all lines into a single String with \n between them
    let mut content = String::new();
    for (i, line) in lines.iter().enumerate() {
        content.push_str(line);
        // Add \n between lines, but not after the last one
        if i + 1 < lines.len() {
            content.push('\n');
        }
    }

    // Write to file (creates or overwrites)
    fs::write(path, content)?;

    Ok(())
}
