use std::{
    collections::btree_map::Values,
    env::{self, var},
    fs,
};

pub fn echo(args: &Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        println!();
        return Ok(());
    }

    let mut output_words = Vec::new();

    for arg in args {
        if arg == "*" {
            // should print all the content of the current directory
            if let Ok(elements) = fs::read_dir(".") {
                let mut files = Vec::new();
                for element in elements.flatten() {
                    let file_name = element.file_name().to_string_lossy().to_string();

                    if !file_name.starts_with('.') {
                        files.push(file_name);
                    }
                }
                files.sort();
                output_words.extend(files);
            }
        } else if arg.starts_with("$") && arg.len() > 1 {
            let var_name = &arg[1..];
            match env::var(var_name) {
                Ok(value) => output_words.push(value),
                Err(_) => {}
            }
        } else {
            output_words.push(arg.clone());
        }
    }

    println!("{}", output_words.join(" "));

    Ok(())
}
