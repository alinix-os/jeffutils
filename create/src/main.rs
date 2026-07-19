
pub mod dir;
pub mod file;

use std::str::FromStr;

use crate::file::{create, create_with_content};


#[derive(Debug, PartialEq)]
enum ARGS {
    FILE, // --file / -f
    DIR, // --dir / -d
    RECURSIVE, // --recursive / -rec / -r
    CONTENT, // --content / -c
    CONTENTTEXT(String), // <content text>
    HELP, // --help / -h
}

impl FromStr for ARGS {
    type Err = String;

    fn from_str(s: &str) -> Result<ARGS, String> {
        match s {
            "--file" | "-f" => Ok(ARGS::FILE),
            "--dir" | "-d" => Ok(ARGS::DIR),
            "--recursive" | "-rec" | "-r" => Ok(ARGS::RECURSIVE),
            "--content" | "-c" => Ok(ARGS::CONTENT),
            "--help" | "-h" => Ok(ARGS::HELP),
            _ => Ok(ARGS::CONTENTTEXT(s.to_string())),
        }
    }
}



fn print_usage() {
    eprintln!("Usage: {} <destino> [-f|-d] [-r] [-c <content>]", std::env::args().nth(0).unwrap_or_else(|| "command".into()));
}

fn find_matching_brace(s: &str, start: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            _ => {}
        }
    }
    None
}

fn expand_braces(s: &str) -> Vec<String> {
    if let Some(start) = s.find('{') {
        if let Some(end) = find_matching_brace(s, start) {
            let prefix = &s[..start];
            let middle = &s[start+1..end];
            let suffix = &s[end+1..];
            let mut result = Vec::new();
            let mut brace_depth = 0;
            let mut current = String::new();
            for c in middle.chars() {
                match c {
                    '{' => {
                        brace_depth += 1;
                        current.push(c);
                    }
                    '}' => {
                        brace_depth -= 1;
                        current.push(c);
                    }
                    ',' if brace_depth == 0 => {
                        let expanded = format!("{}{}{}", prefix, current, suffix);
                        result.extend(expand_braces(&expanded));
                        current.clear();
                    }
                    _ => current.push(c),
                }
            }
            let expanded = format!("{}{}{}", prefix, current, suffix);
            result.extend(expand_braces(&expanded));
            return result;
        }
    }
    vec![s.to_string()]
}

fn main() {
    let argv: Vec<ARGS> = std::env::args().collect::<Vec<String>>()[1..].iter().filter_map(|s| ARGS::from_str(s).ok()).collect();

    if argv.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    if argv.contains(&ARGS::HELP) {
        print_usage();
        println!("Options:");
        println!("  --file, -f             Create a file (default)");
        println!("  --dir, -d              Create a directory");
        println!("  --recursive, -rec, -r  Create intermediate directories");
        println!("  --content, -c          Initial file content");
        println!("  --help, -h             Show this help message");
        std::process::exit(0);
    }

    if argv.contains(&ARGS::FILE) && argv.contains(&ARGS::DIR) {
        eprintln!("Error: cannot use --file/-f and --dir/-d together");
        std::process::exit(1);
    }

    let content_arg = match argv.iter().position(|a| *a == ARGS::CONTENT) {
        Some(content_idx) => match argv.get(content_idx + 1) {
            Some(ARGS::CONTENTTEXT(text)) => Some(text.clone()),
            _ => {
                eprintln!("Error: --content/-c must be followed by content text");
                std::process::exit(1);
            }
        },
        None => None,
    };

    let content_idx = argv.iter().position(|a| *a == ARGS::CONTENT);
    let path_text = argv
        .iter()
        .enumerate()
        .find_map(|(i, a)| match a {
            ARGS::CONTENTTEXT(text) if Some(i) != content_idx.map(|ci| ci + 1) => Some(text.clone()),
            _ => None,
        });

    let path_text = match path_text {
        Some(path) => path,
        None => {
            print_usage();
            std::process::exit(1);
        }
    };

    let paths = expand_braces(&path_text);
    let contents = content_arg.as_ref().map(|c| expand_braces(c)).unwrap_or_else(|| vec![String::new()]);

    if paths.len() > 1 && contents.len() > 1 && paths.len() != contents.len() {
        eprintln!("Error: number of paths ({}) does not match number of contents ({})", paths.len(), contents.len());
        std::process::exit(1);
    }

    let recursive = argv.contains(&ARGS::RECURSIVE);
    let is_dir = argv.contains(&ARGS::DIR);

    for (i, path) in paths.iter().enumerate() {
        let content = if contents.len() > 1 {
            &contents[i]
        } else {
            &contents[0]
        };

        if is_dir {
            dir::mkdir(path, recursive);
        } else if content.is_empty() && content_arg.is_none() {
            create(path, recursive);
        } else {
            create_with_content(path, content, recursive);
        }
    }
}
