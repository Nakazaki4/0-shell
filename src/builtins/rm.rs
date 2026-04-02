use std::fs;
use std::path::Path;

pub fn remove(args: &Vec<String>) {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return;
    }

    let mut recursive = false;
    let mut paths = Vec::new();

    for arg in args {
        if arg == "-r" {
            recursive = true;
        } else {
            paths.push(arg);
        }
    }

    for path in paths {
        let p = Path::new(&path);
        let res = if recursive {
            fs::remove_dir_all(p)
        } else {
            fs::remove_file(p)
        };

        if let Err(e) = res {
            eprintln!("rm: cannot remove {}: {}", path, e);
        }
    }
}