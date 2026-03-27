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

        let mut full_input = String::new();

        loop {
            let mut line = String::new();

            match io::stdin().read_line(&mut line) {
                Ok(0) => {
                    if full_input.is_empty() {
                        println!("exit");
                        break 'shell;
                    }
                    break;
                }
                Ok(_) => {
                    let trimmed_line = line.trim_end();
                    if trimmed_line.ends_with('\\') {
                        let without_slash = &line[..trimmed_line.len() - 1];
                        full_input.push_str(without_slash);

                        print!("> ");
                        if let Err(e) = io::stdout().flush() {
                            eprintln!("Failed to flush stdout: {}", e);
                            continue;
                        }
                    } else {
                        full_input.push_str(&line);
                        break;
                    }
                }
                Err(error) => {
                    eprintln!("Error reading input: {}", error);
                }
            }
        }

        if full_input.trim().len() == 0 {
            continue;
        }

        let trimmed_command = full_input.trim();
        let tokenized_command = tokenize(trimmed_command);
        let command_tree = parse(tokenized_command);
        match execute(command_tree.unwrap()) {
            Ok(_) => {}
            Err(e) => eprintln!("{}", e),
        }
    }
}
