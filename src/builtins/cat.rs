use std::{
    fs::File,
    io::{self},
};

pub fn concatenate(args: &Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        let mut stdin = io::stdin();
        let mut stdout = io::stdout();

        if let Err(e) = io::copy(&mut stdin, &mut stdout) {
            return Err(format!("cat: error reading stdin: {}", e));
        }
        return Ok(());
    }

    for filename in args {
        match File::open(filename) {
            Ok(mut file) => {
                let mut stdout = io::stdout();
                if let Err(e) = io::copy(&mut file, &mut stdout) {
                    // Return the error immediately if reading/writing fails
                    return Err(format!("cat: {}: {}", filename, e));
                }
            }
            Err(e) => {
                // Return the error immediately if the file doesn't exist
                return Err(format!("cat: {}: {}", filename, e));
            }
        }
    }
    
    Ok(())
}