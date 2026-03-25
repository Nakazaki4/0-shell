use std::{fs};

pub fn make_directory(args: &Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err("mkdir: missing operand".to_string());
    }

    for dir_name in args {
        if let Err(e) = fs::create_dir(dir_name) {
            eprintln!("mkdir: cannot create directory '{}': {}", dir_name, e.to_string());
        }
    }
    Ok(())
}
