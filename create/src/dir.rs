use std::fs;

use crate::file::describe_error_kind;

pub fn mkdir(path: &str, recursive: bool) {
    let result = match recursive {
        true => fs::create_dir_all(&path),
        false => fs::create_dir(&path),
    };

    if let Err(e) = result {
        println!("{}", describe_error_kind(e.kind(), "Directory already exists"));
        std::process::exit(1);
    }

    println!("Directory {} created", path);
}
