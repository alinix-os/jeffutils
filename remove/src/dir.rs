use std::fs;

use crate::file::describe_error_kind;

pub fn remove(path: &str, recursive: bool) {
    let result = match recursive {
        true => fs::remove_dir_all(path),
        false => fs::remove_dir(path),
    };

    if let Err(e) = result {
        if e.kind() == std::io::ErrorKind::DirectoryNotEmpty {
            eprintln!("Directory not empty: use -r/--recursive to remove it and its contents");
        } else {
            println!("{}", describe_error_kind(e.kind()));
        }
        std::process::exit(1);
    }

    println!("Directory {} removed", path);
}
