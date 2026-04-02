use std::env;

pub fn change_directory(args: &[String]) -> Result<(), String> {
    if args.len() > 1 {
        return Err("cd: too many arguments".into());
    }

    let mut target = if args.is_empty() {
        env::var("HOME").map_err(|_| "cd: HOME not set".to_string())?
    } else {
        args[0].clone()
    };

    if target.starts_with('~') {
        let home = env::var("HOME").map_err(|_| "cd: HOME not set".to_string())?;
        target = format!("{}{}", home, &target[1..]);
    }

    if let Err(e) = env::set_current_dir(&target) {
        return Err(format!("cd: {}: {}", target, e));
    }

    if let Ok(new_dir) = env::current_dir() {
        unsafe { env::set_var("PWD", new_dir) };
    }

    Ok(())
}
