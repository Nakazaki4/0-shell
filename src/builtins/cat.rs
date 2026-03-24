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
                    eprintln!("cat: {}: {}", filename, e);
                }
            }
            Err(e) => {
                eprintln!("cat: {}: {}", filename, e);
            }
        }
    }
    Ok(())
}
