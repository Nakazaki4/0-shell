use std::path::Path;
use std::{env, fs};

pub fn copy(args: &Vec<String>) -> Result<(), String> {
    if args.len() < 2 {
        return Err("cp: missing file operand".to_string());
    }

    let mut target_str = args.last().cloned().unwrap();
    if target_str.starts_with('~') {
        let home = env::var("HOME").map_err(|_| "cd: HOME not set".to_string())?;
        target_str = format!("{}/{}", home, &target_str[1..]);
    }

    let target_path = Path::new(&target_str);
    let sources = &args[..args.len() - 1];

    if sources.len() > 1 && !target_path.is_dir() {
        return Err(format!("cp: target '{}' is not a directory", target_str));
    }

    for source_str in sources {
        let source_path = Path::new(source_str);

        let destination = if target_path.is_dir() {
            match source_path.file_name() {
                Some(name) => target_path.join(name),
                None => {
                    eprintln!("cp: omitting directory '{}'", source_str);
                    continue;
                }
            }
        } else {
            target_path.to_path_buf()
        };

        if let Err(e) = fs::copy(&source_path, &destination) {
            eprintln!("cp: cannot create regular file '{}': {}", target_str, e);
        }
    }

    Ok(())
}
