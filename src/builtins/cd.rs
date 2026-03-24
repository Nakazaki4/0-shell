use std::env;

pub fn change_directory(args: &[String]) -> Result<(), String> {
    if args.len() > 1 {
        return Err("cd: too many arguments".into());
    }

    let target = if args.is_empty() {
        env::var("HOME").map_err(|_| "cd: HOME not set".to_string())?
    } else {
        args[0].clone()
    };

    env::set_current_dir(&target).map_err(|e| format!("cd: {}: {}", target, e))
}
