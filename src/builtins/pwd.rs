use std::env;

pub fn print_working_dir(args: &Vec<String>) -> Result<(), String>{
    let curr_dir = env::current_dir();
    let current_path = match curr_dir {
        Ok(path_buf)=> path_buf,
        Err(error) => {
                return Err(format!("Failed to get current directory {}", error));
            }

    };
    println!("{}", current_path.display());
    Ok(())
}