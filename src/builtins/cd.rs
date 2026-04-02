use std::env;

pub fn change_directory(args: &[String]) -> Result<(), String> {
    if args.len() > 1 {
        return Err("cd: too many arguments".into());
    }

    let old_dir = env::var("PWD").unwrap_or_else(|_| {
        env::current_dir().map(|p| p.display().to_string()).unwrap_or_default()
    });

    let mut target = if args.is_empty() {
        env::var("HOME").map_err(|_| "cd: HOME not set".to_string())?
    } else if args[0] == "-" {
        env::var("OLDPWD").map_err(|_| "cd: OLDPWD not set".to_string())?
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

    if !args.is_empty() && args[0] == "-" {
        println!("{}", target);
    }

    unsafe {
        if !old_dir.is_empty() {
            env::set_var("OLDPWD", old_dir);
        }
        if let Ok(new_dir) = env::current_dir() {
            env::set_var("PWD", new_dir);
        }
    }

    Ok(())
}