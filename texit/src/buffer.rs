// buffer module — manages text storage, cursor position, and editing operations

use std::path::PathBuf;

/// Main structure that holds the text content and editor state
pub struct Buffer {
    /// Each line of text is stored as a String
    pub lines: Vec<String>,
    /// Absolute column of the cursor (index within the current line)
    pub cx: usize,
    /// Absolute row of the cursor (index in the lines vector)
    pub cy: usize,
    /// Horizontal scroll offset (how many columns scrolled to the right)
    pub scroll_x: usize,
    /// Vertical scroll offset (how many lines scrolled up)
    pub scroll_y: usize,
    /// Path to the current file (None if it is a new unnamed buffer)
    pub filepath: Option<PathBuf>,
    /// True if the buffer has been modified since the last save
    pub modified: bool,
}

impl Buffer {
    /// Creates an empty buffer, optionally associated with a file path
    pub fn new(filepath: Option<PathBuf>) -> Self {
        Self {
            lines: vec![String::new()],
            cx: 0,
            cy: 0,
            scroll_x: 0,
            scroll_y: 0,
            filepath,
            modified: false,
        }
    }

    /// Creates a buffer from existing lines and an optional file path
    pub fn with_lines(lines: Vec<String>, filepath: Option<PathBuf>) -> Self {
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Self {
            lines,
            cx: 0,
            cy: 0,
            scroll_x: 0,
            scroll_y: 0,
            filepath,
            modified: false,
        }
    }

    /// Inserts a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cy];
        let byte_pos = line.char_indices().nth(self.cx).map(|(i, _)| i).unwrap_or(line.len());
        line.insert(byte_pos, c);
        self.cx += 1;
        self.modified = true;
    }

    /// Deletes the character before the cursor (Backspace)
    pub fn delete_char(&mut self) {
        if self.cx > 0 {
            // Delete within the same line
            self.cx -= 1;
            let byte_pos = self.lines[self.cy].char_indices().nth(self.cx).map(|(i, _)| i).unwrap_or(self.lines[self.cy].len());
            self.lines[self.cy].remove(byte_pos);
            self.modified = true;
        } else if self.cy > 0 {
            // Merge current line with the previous one
            let current_line = self.lines.remove(self.cy);
            self.cy -= 1;
            self.cx = self.lines[self.cy].chars().count();
            self.lines[self.cy].push_str(&current_line);
            self.modified = true;
        }
    }

    /// Deletes the character under the cursor (Delete)
    pub fn delete_fwd(&mut self) {
        let len = self.lines[self.cy].chars().count();
        if self.cx < len {
            // Remove the character under the cursor
            let byte_pos = self.lines[self.cy].char_indices().nth(self.cx).map(|(i, _)| i).unwrap_or(self.lines[self.cy].len());
            self.lines[self.cy].remove(byte_pos);
            self.modified = true;
        } else if self.cy + 1 < self.lines.len() {
            // Merge with the next line
            let next = self.lines.remove(self.cy + 1);
            self.lines[self.cy].push_str(&next);
            self.modified = true;
        }
    }

    /// Inserts a newline at the cursor position (Enter)
    pub fn insert_newline(&mut self) {
        let line = &mut self.lines[self.cy];
        let byte_pos = line.char_indices().nth(self.cx).map(|(i, _)| i).unwrap_or(line.len());
        let rest = line.split_off(byte_pos);
        self.lines.insert(self.cy + 1, rest);
        self.cy += 1;
        self.cx = 0;
        self.modified = true;
    }

    /// Moves the cursor up one line
    pub fn move_up(&mut self) {
        if self.cy > 0 {
            self.cy -= 1;
            self.clamp_cx_to_line();
        }
    }

    /// Moves the cursor down one line
    pub fn move_down(&mut self) {
        if self.cy + 1 < self.lines.len() {
            self.cy += 1;
            self.clamp_cx_to_line();
        }
    }

    /// Moves the cursor left by one character
    pub fn move_left(&mut self) {
        if self.cx > 0 {
            self.cx -= 1;
        } else if self.cy > 0 {
            // Move to the end of the previous line
            self.cy -= 1;
            self.cx = self.lines[self.cy].chars().count();
        }
    }

    /// Moves the cursor right by one character
    pub fn move_right(&mut self) {
        let len = self.lines[self.cy].chars().count();
        if self.cx < len {
            self.cx += 1;
        } else if self.cy + 1 < self.lines.len() {
            // Move to the start of the next line
            self.cy += 1;
            self.cx = 0;
        }
    }

    /// Moves the cursor to the beginning of the line (Home)
    pub fn go_home(&mut self) {
        self.cx = 0;
    }

    /// Moves the cursor to the end of the line (End)
    pub fn go_end(&mut self) {
        self.cx = self.lines[self.cy].chars().count();
    }

    /// Moves the cursor up by one screenful (PageUp)
    pub fn page_up(&mut self, screen_lines: usize) {
        for _ in 0..screen_lines {
            if self.cy == 0 {
                break;
            }
            self.cy -= 1;
        }
        self.clamp_cx_to_line();
    }

    /// Moves the cursor down by one screenful (PageDown)
    pub fn page_down(&mut self, screen_lines: usize) {
        for _ in 0..screen_lines {
            if self.cy + 1 >= self.lines.len() {
                break;
            }
            self.cy += 1;
        }
        self.clamp_cx_to_line();
    }

    /// Ensures the cursor does not go beyond buffer boundaries
    pub fn ensure_cursor_in_bounds(&mut self) {
        if self.cy >= self.lines.len() {
            self.cy = self.lines.len().saturating_sub(1);
        }
        let len = self.lines[self.cy].chars().count();
        if self.cx > len {
            self.cx = len;
        }
    }

    /// Adjusts scroll offsets so the cursor is always visible on screen
    pub fn ensure_cursor_visible(&mut self, term_cols: usize, term_rows: usize) {
        // Vertical scroll
        if self.cy < self.scroll_y {
            self.scroll_y = self.cy;
        } else if self.cy >= self.scroll_y + term_rows {
            self.scroll_y = self.cy - term_rows + 1;
        }

        // Horizontal scroll
        if self.cx < self.scroll_x {
            self.scroll_x = self.cx;
        } else if self.cx >= self.scroll_x + term_cols {
            self.scroll_x = self.cx - term_cols + 1;
        }
    }

    /// Clamps cx so it does not exceed the current line length
    fn clamp_cx_to_line(&mut self) {
        let len = self.lines[self.cy].chars().count();
        if self.cx > len {
            self.cx = len;
        }
    }

    /// Searches forward from the cursor position for the given query string
    ///
    /// Returns `Some((line, col))` of the first match, or `None` if not found.
    pub fn search_forward(&self, query: &str) -> Option<(usize, usize)> {
        if query.is_empty() {
            return None;
        }

        // Search from current cursor position in the current line
        let byte_pos = self.lines[self.cy].char_indices().nth(self.cx).map(|(i, _)| i).unwrap_or(self.lines[self.cy].len());
        if let Some(byte_offset) = self.lines[self.cy][byte_pos..].find(query) {
            let byte_end = byte_pos + byte_offset;
            let col = self.lines[self.cy][..byte_end].chars().count();
            return Some((self.cy, col));
        }

        // Search remaining lines
        for y in (self.cy + 1)..self.lines.len() {
            if let Some(col) = self.lines[y].find(query) {
                return Some((y, col));
            }
        }

        // Not found
        None
    }
}
