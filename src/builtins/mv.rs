use std::path::Path;
use std::{env, fs};

pub fn movee(args: &Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err("mv: missing file operand".to_string());
    } else if args.len() == 1 {
        return Err(format!(
            "mv: missing destination file operand after '{}'",
            args[0]
        ));
    }

    let mut target_str = args.last().cloned().unwrap();
    if target_str.starts_with('~') {
        let home = env::var("HOME").map_err(|_| "mv: HOME not set".to_string())?;
        target_str = format!("{}{}", home, &target_str[1..]);
    }

    let target_path = Path::new(&target_str);
    let sources = &args[..args.len() - 1];

    if sources.len() > 1 && !target_path.is_dir() {
        return Err(format!("mv: target '{}' is not a directory", target_str));
    }

    for source_raw in sources {
        let mut source_str = source_raw.clone();
        if source_str.starts_with('~') {
            if let Ok(home) = env::var("HOME") {
                source_str = format!("{}{}", home, &source_str[1..]);
            }
        }

        let source_path = Path::new(&source_str);

        let destination = if target_path.is_dir() {
            match source_path.file_name() {
                Some(name) => target_path.join(name),
                None => {
                    eprintln!("mv: omitting directory '{}'", source_str);
                    continue;
                }
            }
        } else {
            target_path.to_path_buf()
        };

        if let Err(e) = fs::rename(&source_path, &destination) {
            eprintln!(
                "mv: cannot stat '{}': No such file or directory",
                source_str,
            );
        }
    }
    Ok(())
}
