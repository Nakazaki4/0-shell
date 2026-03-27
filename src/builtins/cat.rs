use std::{
    fs::File,
    io::{self},
};

pub fn concatenate(args: &Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        let mut stdin = io::stdin();
        let mut stdout = io::stdout();

        if let Err(e) = io::copy(&mut stdin, &mut stdout) {
            let err_msg = e.to_string();
            let clean_err = err_msg.split(" (os error)").next().unwrap_or(&err_msg);
            return Err(format!("cat: error reading stdin: {}", clean_err));
        }
        return Ok(());
    }

    for filename in args {
        match File::open(filename) {
            Ok(mut file) => {
                let mut stdout = io::stdout();
                if let Err(e) = io::copy(&mut file, &mut stdout) {
                    // Return the error immediately if reading/writing fails
                    let err_msg = e.to_string();
                    let clean_err = err_msg.split(" (os error)").next().unwrap_or(&err_msg);
                    return Err(format!("cat: {}: {}", filename, clean_err));
                }
            }
            Err(e) => {
                // Return the error immediately if the file doesn't exist
                let err_msg = e.to_string();
                let clean_err = err_msg.split(" (os error)").next().unwrap_or(&err_msg);
                return Err(format!("cat: {}: {}", filename, clean_err));
            }
        }
    }

    Ok(())
}
