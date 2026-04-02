use std::env;

pub fn print_working_dir(_args: &Vec<String>) -> Result<(), String> {
    if let Ok(logical_path) = env::var("PWD") {
        println!("{}", logical_path);
        return Ok(());
    }

    let curr_dir = env::current_dir();
    let current_path = match curr_dir {
        Ok(path_buf) => path_buf,
        Err(error) => {
            return Err(format!("Failed to get current directory {}", error));
        }
    };
    
    println!("{}", current_path.display());
    Ok(())
}