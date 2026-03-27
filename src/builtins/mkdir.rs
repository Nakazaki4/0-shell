use std::fs;

pub fn make_directory(args: &Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err("mkdir: missing operand".to_string());
    }

    for dir_name in args {
        if let Err(e) = fs::create_dir(dir_name) {
            let err_msg = e.to_string();
            let clean_err = err_msg.split(" (os error)").next().unwrap_or(&err_msg);
            eprintln!(
                "mkdir: cannot create directory '{}': {}",
                dir_name, clean_err
            );
        }
    }
    Ok(())
}
