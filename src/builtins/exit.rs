pub fn exit(args: &Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        std::process::exit(0);
    }

    match args[0].parse::<i32>() {
        Ok(code) => {
            std::process::exit(code);
        }
        Err(_) => {
            eprintln!("0-shell: exit: {}: numeric argument required", args[0]);
            std::process::exit(2);
        }
    }
}
