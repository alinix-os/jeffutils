// texit — Native text editor for JeffNix / Alinix Distro
//
// Usage: texit [path/to/file]
//
// A minimal terminal-based text editor similar to nano,
// with arrow key navigation, basic editing, Ctrl+S to save, Ctrl+X to quit,
// and Ctrl+W to search.

mod buffer;
mod fileio;
mod input;
mod render;

use std::io::{stdout, Write};
use std::path::PathBuf;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::QueueableCommand;

use buffer::Buffer;
use render::Render;

/// Prints usage information to stderr
fn print_usage() {
    eprintln!("Jefferson S. Rios, texit 1.0 | 2026");
    eprintln!();
    eprintln!("Usage: texit [path/to/file]");
    eprintln!();
    eprintln!("A terminal-based text editor inspired by nano.");
    eprintln!();
    eprintln!("Keys:");
    eprintln!("  Ctrl+S    Save file");
    eprintln!("  Ctrl+X    Quit");
    eprintln!("  Ctrl+W    Search");
    eprintln!("  Arrows    Navigate");
    eprintln!("  Home/End  Beginning/End of line");
    eprintln!("  PgUp/Dn   Page up/down");
}

/// Prints the program version
fn print_version() {
    eprintln!("texit version 0.1.0");
}

/// Detects if the system locale is Portuguese
fn locale_is_portuguese() -> bool {
    let lang = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"));
    match lang {
        Ok(val) => val.to_lowercase().starts_with("pt"),
        Err(_) => false,
    }
}

/// Reads a line of input from the user in raw mode, showing a prompt on the
/// status line. Returns `Some(input)` on Enter, or `None` on Esc.
fn prompt_input(
    render: &mut Render,
    stdout: &mut impl Write,
    buffer: &Buffer,
    prompt: &str,
) -> Option<String> {
    let mut input = String::new();

    loop {
        // Update the status message with the prompt and current input
        let msg = format!("{}{}", prompt, input);
        render.status_message = Some(msg);
        render.draw_screen(stdout, buffer);

        let key = input::read_key();

        match key {
            // Enter — confirm input
            input::Key::Enter => {
                break Some(input);
            }
            // Esc — cancel
            input::Key::Esc | input::Key::Ctrl('x') => {
                break None;
            }
            // Backspace — remove last character
            input::Key::Backspace => {
                input.pop();
            }
            // Regular character — append to input
            input::Key::Char(c) => {
                // Ignore control characters
                if !c.is_control() {
                    input.push(c);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Handle --help and --version flags
    for arg in &args[1..] {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--version" | "-v" => {
                print_version();
                return;
            }
            _ => {}
        }
    }

    // Load the file or create an empty buffer
    let mut buffer = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);

        if path.to_string_lossy() == "--help" || path.to_string_lossy() == "-h" {
            print_usage();
            return;
        }
        if path.to_string_lossy() == "--version" || path.to_string_lossy() == "-v" {
            print_version();
            return;
        }

        match fileio::read_file(&path) {
            Ok(lines) => Buffer::with_lines(lines, Some(path)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Buffer::new(Some(path))
            }
            Err(e) => {
                eprintln!("texit: error opening '{}': {}", args[1], e);
                std::process::exit(1);
            }
        }
    } else {
        Buffer::new(None)
    };

    let mut stdout = stdout();

    // Enable raw terminal mode
    if let Err(e) = enable_raw_mode() {
        eprintln!("texit: error entering raw mode: {}", e);
        std::process::exit(1);
    }

    // Switch to alternate screen buffer (like nano — restores original screen on exit)
    let _ = execute!(stdout, EnterAlternateScreen);

    // Detect locale once at startup
    let is_pt = locale_is_portuguese();

    // Get the initial terminal size
    let (term_cols, term_rows) = size().unwrap_or((80, 24));
    let mut render = Render::new(term_cols as usize, term_rows as usize);

    // Main editing loop
    loop {
        // Text area: total minus header bar(1), status line(1), shortcut bar(1)
        let text_rows = render.term_rows.saturating_sub(3);

        buffer.ensure_cursor_in_bounds();
        buffer.ensure_cursor_visible(render.term_cols, text_rows);

        // Clear any persistent status message before drawing
        render.status_message = None;
        render.draw_screen(&mut stdout, &buffer);

        // Read the next key press
        let key = input::read_key();

        match key {
            // Ctrl+S — Save file
            input::Key::Ctrl('s') => {
                if let Some(path) = &buffer.filepath.clone() {
                    match fileio::save_file(path, &buffer.lines) {
                        Ok(()) => {
                            buffer.modified = false;
                        }
                        Err(e) => {
                            let _ = writeln!(stdout, "\r\ntexit: error saving: {}\r\n", e);
                            let _ = stdout.flush();
                        }
                    }
                }
            }
            // Ctrl+X — Quit the editor (with save prompt if modified)
            input::Key::Ctrl('x') => {
                if buffer.modified {
                    let prompt = if is_pt {
                        "Salvar buffer modificado? (S/N/C)"
                    } else {
                        "Save modified buffer? (Y/N/C)"
                    };
                    render.status_message = Some(prompt.into());
                    render.draw_screen(&mut stdout, &buffer);

                    match input::read_key() {
                        // Y/S = save and quit
                        input::Key::Char('y') | input::Key::Char('Y')
                        | input::Key::Char('s') | input::Key::Char('S') => {
                            if let Some(path) = &buffer.filepath.clone() {
                                let _ = fileio::save_file(path, &buffer.lines);
                            }
                            break;
                        }
                        // N = discard and quit
                        input::Key::Char('n') | input::Key::Char('N') => break,
                        // C or any other key = cancel (stay in editor)
                        _ => {}
                    }
                } else {
                    break;
                }
            }
            // Ctrl+W — Search
            input::Key::Ctrl('w') => {
                let search_term = prompt_input(&mut render, &mut stdout, &buffer, "Search: ");
                if let Some(ref query) = search_term {
                    if !query.is_empty() {
                        if let Some((line, col)) = buffer.search_forward(query) {
                            buffer.cy = line;
                            buffer.cx = col;
                        }
                    }
                }
            }
            // Ctrl+G — Help (show help overlay, then wait for a key)
            input::Key::Ctrl('g') => {
                // Briefly show help info on the status line
                render.status_message = Some(
                    "^S Save  ^X Quit  ^W Search  ^O Write As  ^G Help  Press any key...".into()
                );
                render.draw_screen(&mut stdout, &buffer);
                input::read_key();
            }
            // Regular character — insert into the buffer
            input::Key::Char(c) => buffer.insert_char(c),
            // Enter — split the line
            input::Key::Enter => buffer.insert_newline(),
            // Backspace — delete previous character
            input::Key::Backspace => buffer.delete_char(),
            // Delete — delete character under cursor
            input::Key::Delete => buffer.delete_fwd(),
            // Arrow key navigation
            input::Key::Up => buffer.move_up(),
            input::Key::Down => buffer.move_down(),
            input::Key::Left => buffer.move_left(),
            input::Key::Right => buffer.move_right(),
            // Home / End
            input::Key::Home => buffer.go_home(),
            input::Key::End => buffer.go_end(),
            // PageUp / PageDown
            input::Key::PageUp => buffer.page_up(text_rows),
            input::Key::PageDown => buffer.page_down(text_rows),
            // Tab — insert 4 spaces
            input::Key::Tab => {
                for _ in 0..4 {
                    buffer.insert_char(' ');
                }
            }
            // Ctrl+O — Write As (save to a different path)
            input::Key::Ctrl('o') => {
                let path_str = prompt_input(&mut render, &mut stdout, &buffer, "Write As: ");
                if let Some(ref path_str) = path_str {
                    if !path_str.is_empty() {
                        let new_path = PathBuf::from(path_str);
                        match fileio::save_file(&new_path, &buffer.lines) {
                            Ok(()) => {
                                buffer.filepath = Some(new_path);
                                buffer.modified = false;
                            }
                            Err(e) => {
                                let _ = writeln!(stdout, "\r\ntexit: error saving: {}\r\n", e);
                                let _ = stdout.flush();
                            }
                        }
                    }
                }
            }
            // Ignore unknown keys
            _ => {}
        }

        // Check if the terminal has been resized
        if let Ok((new_cols, new_rows)) = size() {
            if new_cols as usize != render.term_cols || new_rows as usize != render.term_rows {
                render.term_cols = new_cols as usize;
                render.term_rows = new_rows as usize;
            }
        }
    }

    // Leave alternate screen (restores original terminal content) and exit raw mode
    let _ = stdout.queue(Show);
    let _ = stdout.flush();
    let _ = execute!(stdout, LeaveAlternateScreen);

    if let Err(e) = disable_raw_mode() {
        eprintln!("texit: error leaving raw mode: {}", e);
        std::process::exit(1);
    }
}
