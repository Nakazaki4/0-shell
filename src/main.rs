// Lexer -> Parser -> Executor
mod builtins;
mod executor;
mod lexer;
mod parser;

use lexer::tokenize;
use parser::parse;
use std::env;
use std::io::{self, Write};

use crate::executor::execute;

fn main() {
    'shell: loop {
        let curr_dir = env::current_dir();
        let current_path = match curr_dir {
            Err(error) => {
                eprintln!("Failed to get current directory {}", error);
                return;
            }
            Ok(path_buf) => path_buf,
        };

        print!("{} $ ", current_path.display());

        if let Err(e) = io::stdout().flush() {
            eprintln!("Failed to flush stdout: {}", e);
            continue;
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {
                let trimmed_command = input.trim();
                let tokenized_command = tokenize(trimmed_command); // tokenization
                let command_tree = parse(tokenized_command);
                match execute(command_tree.unwrap()) {
                    Ok(_) => {}
                    Err(e) => eprintln!("{}", e),
                }
            }
            Err(error) => {
                eprintln!("Error reading input: {}", error);
            }
        }

        // show the current directory ($ ~/Desktop/)
        // pass the user input to the lexer
    }
}
