// input module — raw mode key reading and interpretation

use std::io::{Read, stdin};

/// Enum representing all recognized key presses
#[derive(Debug, PartialEq)]
pub enum Key {
    /// A regular character key (letter, number, symbol)
    Char(char),
    /// Ctrl + letter combination (Ctrl+A through Ctrl+Z)
    Ctrl(char),
    /// Enter key
    Enter,
    /// Tab key
    Tab,
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Home key
    Home,
    /// End key
    End,
    /// PageUp key
    PageUp,
    /// PageDown key
    PageDown,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Escape key
    Esc,
    /// Any unrecognized sequence
    Unknown,
}

/// Reads a single key press from stdin in raw mode and converts it to a Key
pub fn read_key() -> Key {
    let mut buf = [0u8; 1];

    // Read the first byte; return Unknown on failure
    if stdin().read(&mut buf).ok() != Some(1) {
        return Key::Unknown;
    }

    match buf[0] {
        // Enter (both \n and \r)
        b'\n' | b'\r' => Key::Enter,
        // Tab
        b'\t' => Key::Tab,
        // Backspace (ASCII DEL)
        b'\x7f' => Key::Backspace,
        // Escape — could be a lone Esc or the start of an escape sequence
        b'\x1b' => read_escape_sequence(),
        // Ctrl+A through Ctrl+Z (bytes 1 to 26)
        b if b < 32 => {
            let c = (b + 96) as char;
            Key::Ctrl(c)
        }
        // Regular character
        c => Key::Char(c as char),
    }
}

/// Interprets escape sequences starting with \x1b
///
/// Common sequences:
///   ESC [ A  →  Up
///   ESC [ B  →  Down
///   ESC [ C  →  Right
///   ESC [ D  →  Left
///   ESC [ H  →  Home
///   ESC [ F  →  End
///   ESC [ 3 ~  →  Delete
///   ESC [ 5 ~  →  PageUp
///   ESC [ 6 ~  →  PageDown
fn read_escape_sequence() -> Key {
    let mut buf = [0u8; 1];

    // Read the next byte after ESC
    if stdin().read(&mut buf).ok() != Some(1) {
        return Key::Esc;
    }

    match buf[0] {
        // CSI sequences (Control Sequence Introducer): ESC [
        b'[' => read_csi_sequence(),
        // SS3 sequences: ESC O (used by Home/End on some terminals)
        b'O' => read_ss3_sequence(),
        // Lone ESC
        _ => Key::Esc,
    }
}

/// Reads a complete CSI sequence: ESC [ <params...> <letter>
fn read_csi_sequence() -> Key {
    let mut seq = Vec::new();

    // Keep reading bytes until a letter or '~' is found
    loop {
        let mut buf = [0u8; 1];
        if stdin().read(&mut buf).ok() != Some(1) {
            return Key::Unknown;
        }
        seq.push(buf[0]);

        // Sequence ends at a letter (A-Z, a-z) or '~'
        if buf[0].is_ascii_alphabetic() || buf[0] == b'~' {
            break;
        }
    }

    // Match the collected sequence
    match seq.as_slice() {
        [b'A'] => Key::Up,
        [b'B'] => Key::Down,
        [b'C'] => Key::Right,
        [b'D'] => Key::Left,
        [b'H'] => Key::Home,
        [b'F'] => Key::End,
        [b'2', b'~'] => Key::Home,     // Insert, treated as Home
        [b'3', b'~'] => Key::Delete,
        [b'5', b'~'] => Key::PageUp,
        [b'6', b'~'] => Key::PageDown,
        _ => Key::Unknown,
    }
}

/// Reads an SS3 sequence: ESC O <letter>
fn read_ss3_sequence() -> Key {
    let mut buf = [0u8; 1];
    if stdin().read(&mut buf).ok() != Some(1) {
        return Key::Unknown;
    }
    match buf[0] {
        b'H' => Key::Home,
        b'F' => Key::End,
        _ => Key::Unknown,
    }
}
