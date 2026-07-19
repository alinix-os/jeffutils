// render module — draws the buffer content onto the terminal screen

use std::io::Write;
use crossterm::cursor::{MoveTo, Show, Hide};
use crossterm::style::{Print, SetAttribute, Attribute};
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand;

use crate::buffer::Buffer;

/// Handles drawing header bar, text content, status line, shortcut bar, and cursor
pub struct Render {
    /// Number of columns (width) of the terminal
    pub term_cols: usize,
    /// Number of rows (height) of the terminal
    pub term_rows: usize,
    /// Optional custom message shown on the status line (e.g. search prompt)
    pub status_message: Option<String>,
}

impl Render {
    /// Creates a new renderer with the given terminal dimensions
    pub fn new(term_cols: usize, term_rows: usize) -> Self {
        Self {
            term_cols,
            term_rows,
            status_message: None,
        }
    }

    /// Pads a string with spaces to exactly `width` columns, truncating if needed
    fn pad(&self, s: &str) -> String {
        let len = s.chars().count();
        if len >= self.term_cols {
            s.chars().take(self.term_cols).collect()
        } else {
            let mut out = String::with_capacity(self.term_cols);
            out.push_str(s);
            for _ in 0..(self.term_cols - len) {
                out.push(' ');
            }
            out
        }
    }

    /// Draws the full screen: header bar + content + status line + shortcut bar + cursor
    pub fn draw_screen(&self, stdout: &mut impl Write, buffer: &Buffer) {
        // Lines available for text: total minus header(1), status(1), shortcut(1)
        let content_lines = self.term_rows.saturating_sub(3);

        // Hide the cursor while rendering
        let _ = stdout.queue(Hide);

        // ── Header bar at row 0 ──
        self.draw_header_bar(stdout, buffer);

        // ── Content area starting at row 1 ──
        for y in 0..content_lines {
            let screen_row = (y + 1) as u16; // skip header row
            let idx = buffer.scroll_y + y;

            let _ = stdout.queue(MoveTo(0, screen_row));

            if idx < buffer.lines.len() {
                let line = &buffer.lines[idx];
                let skip = buffer.scroll_x;
                let display: String = line.chars().skip(skip).take(self.term_cols).collect();
                let highlighted = highlight_line(&display);
                let _ = stdout.queue(Print(highlighted));
            }

            let _ = stdout.queue(Clear(ClearType::UntilNewLine));
        }

        // ── Status line at second-to-last row ──
        let status_y = self.term_rows.saturating_sub(2) as u16;
        self.draw_status_line(stdout, status_y);

        // ── Shortcut bar at the very bottom ──
        let shortcut_y = self.term_rows.saturating_sub(1) as u16;
        self.draw_shortcut_bar(stdout, shortcut_y);

        // ── Place cursor (content starts at row 1) ──
        let cursor_x = buffer.cx.saturating_sub(buffer.scroll_x) as u16;
        let cursor_y = (buffer.cy.saturating_sub(buffer.scroll_y) + 1) as u16;
        let _ = stdout.queue(MoveTo(cursor_x, cursor_y));
        let _ = stdout.queue(Show);

        let _ = stdout.flush();
    }

    /// Draws the header bar at row 0 — full width reverse video
    fn draw_header_bar(&self, stdout: &mut impl Write, buffer: &Buffer) {
        let filename = buffer
            .filepath
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "[New File]".to_string());

        let modified = if buffer.modified { " [Modified]" } else { "" };
        let total_lines = buffer.lines.len();
        let col = buffer.cx + 1;

        let info = format!(
            "texit | Jefferson S. Rios | [ {} ]{}  Line {}/{}  Col {}",
            filename, modified, buffer.cy + 1, total_lines, col,
        );

        let _ = stdout.queue(MoveTo(0, 0));
        let _ = stdout.queue(SetAttribute(Attribute::Reverse));
        let _ = stdout.queue(Print(self.pad(&info)));
        let _ = stdout.queue(SetAttribute(Attribute::Reset));
    }

    /// Draws the status line — custom prompt/message or empty, full width
    fn draw_status_line(&self, stdout: &mut impl Write, y: u16) {
        let _ = stdout.queue(MoveTo(0, y));
        let _ = stdout.queue(SetAttribute(Attribute::Reverse));

        let display = match &self.status_message {
            Some(msg) => self.pad(msg),
            None => " ".repeat(self.term_cols),
        };
        let _ = stdout.queue(Print(display));
        let _ = stdout.queue(SetAttribute(Attribute::Reset));
    }

    /// Draws the shortcut key help bar at the bottom — full width
    fn draw_shortcut_bar(&self, stdout: &mut impl Write, y: u16) {
        let _ = stdout.queue(MoveTo(0, y));
        let _ = stdout.queue(SetAttribute(Attribute::Reverse));

        let shortcuts = "^S Save  ^X Quit  ^W Search  ^O Write As  ^G Help";
        let _ = stdout.queue(Print(self.pad(shortcuts)));
        let _ = stdout.queue(SetAttribute(Attribute::Reset));
    }
}

fn highlight_line(line: &str) -> String {
    let keywords = [
        "fn", "let", "mut", "match", "if", "else", "for", "while", "return", "use", "mod",
        "struct", "impl", "pub", "static", "const", "true", "false", "import", "def", "class",
        "print", "echo", "alias", "export", "local"
    ];

    let mut highlighted = String::new();
    let mut chars = line.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '/' {
            chars.next();
            if let Some(&'/') = chars.peek() {
                highlighted.push_str("\x1B[90m//");
                chars.next();
                while let Some(cc) = chars.next() {
                    highlighted.push(cc);
                }
                highlighted.push_str("\x1B[0m");
                break;
            } else {
                highlighted.push('/');
            }
        } else if c == '#' {
            highlighted.push_str("\x1B[90m#");
            chars.next();
            while let Some(cc) = chars.next() {
                highlighted.push(cc);
            }
            highlighted.push_str("\x1B[0m");
            break;
        } else if c == '"' || c == '\'' {
            let quote = c;
            highlighted.push_str("\x1B[32m");
            highlighted.push(quote);
            chars.next();
            let mut escaped = false;
            while let Some(&cc) = chars.peek() {
                highlighted.push(cc);
                chars.next();
                if escaped {
                    escaped = false;
                } else if cc == '\\' {
                    escaped = true;
                } else if cc == quote {
                    break;
                }
            }
            highlighted.push_str("\x1B[0m");
        } else if c.is_ascii_digit() {
            highlighted.push_str("\x1B[36m");
            while let Some(&cc) = chars.peek() {
                if cc.is_ascii_digit() || cc == '.' {
                    highlighted.push(cc);
                    chars.next();
                } else {
                    break;
                }
            }
            highlighted.push_str("\x1B[0m");
        } else if c.is_alphabetic() || c == '_' {
            let mut word = String::new();
            while let Some(&cc) = chars.peek() {
                if cc.is_alphanumeric() || cc == '_' {
                    word.push(cc);
                    chars.next();
                } else {
                    break;
                }
            }
            if keywords.contains(&word.as_str()) {
                highlighted.push_str("\x1B[1;33m");
                highlighted.push_str(&word);
                highlighted.push_str("\x1B[0m");
            } else if !word.is_empty() && word.chars().next().unwrap().is_uppercase() {
                highlighted.push_str("\x1B[1;36m");
                highlighted.push_str(&word);
                highlighted.push_str("\x1B[0m");
            } else {
                highlighted.push_str(&word);
            }
        } else {
            highlighted.push(c);
            chars.next();
        }
    }
    highlighted
}
