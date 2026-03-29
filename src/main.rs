// Lexer -> Parser -> Executor
mod builtins;
mod executor;
mod lexer;
mod parser;

use lexer::tokenize;
use parser::parse;
use std::env;
use std::io::{self, Read, Write};
use termios::{ECHO, ICANON, TCSANOW, Termios, tcsetattr};

use crate::executor::execute;

static DELETE: u8 = 8;
static BACK_SPACE: u8 = 127;
static ESCAPE: u8 = 27;
static BRACKET: u8 = 91;
static UP: u8 = 65;
static DOWN: u8 = 66;
static TAB: u8 = 9;
static CTRL_D: u8 = 4;

fn main() {
    let original_config = switch_to_raw_mode();
    let mut history: Vec<String> = Vec::new();

    loop {
        if let Err(e) = print_prompt() {
            eprintln!("\r\n{}", e);
            break;
        }

        let full_input = match read_command() {
            Some(input) => input,
            None => break,
        };

        let trimmed_command = full_input.trim();

        if trimmed_command.is_empty() {
            continue;
        }

        if history.last().map(|s| s.as_str()) != Some(trimmed_command) {
            history.push(trimmed_command.to_string());
        }

        if trimmed_command == "exit" {
            break;
        }

        let tokenized_command = tokenize(trimmed_command);
        let command_tree = parse(tokenized_command);

        match execute(command_tree.unwrap()) {
            Ok(_) => {}
            Err(e) => eprintln!("\r\n{}", e),
        }
    }

    tcsetattr(0, TCSANOW, &original_config).unwrap();
}

fn print_prompt() -> Result<(), String> {
    let curr_dir =
        env::current_dir().map_err(|e| format!("Failed to get current directory {}", e))?;
    print!("{} $ ", curr_dir.display());
    io::stdout()
        .flush()
        .map_err(|e| format!("Failed to flush stdout: {}", e))?;
    Ok(())
}

fn read_command() -> Option<String> {
    let mut full_input = String::new();
    let mut buffer = [0; 1];

    loop {
        io::stdin().read_exact(&mut buffer).unwrap();
        let byte = buffer[0];

        if byte == b'\r' || byte == b'\n' {
            let trimmed = full_input.trim_end();

            if trimmed.ends_with('\\') {
                let slash_pos = full_input.rfind('\\').unwrap();
                full_input.truncate(slash_pos);

                print!("\r\n> ");
                io::stdout().flush().unwrap();
                continue;
            } else {
                print!("\r\n");
                io::stdout().flush().unwrap();
                return Some(full_input);
            }
        } else if byte == CTRL_D {
            if full_input.is_empty() {
                print!("exit\r\n");
                return None;
            }
        } else if byte == DELETE || byte == BACK_SPACE {
            if full_input.pop().is_some() {
                print!("\x08 \x08");
                io::stdout().flush().unwrap();
            }
        } else if byte == ESCAPE {
            io::stdin().read_exact(&mut buffer).unwrap();
            if buffer[0] == BRACKET {
                io::stdin().read_exact(&mut buffer).unwrap();
                if buffer[0] == UP {
                    // TODO: history up
                } else if buffer[0] == DOWN {
                    // TODO: history down
                }
            }
        } else if byte == TAB {
            // TODO: auto completion
        } else {
            let c = byte as char;
            full_input.push(c);
            print!("{}", c);
            io::stdout().flush().unwrap();
        }
    }
}

fn switch_to_raw_mode() -> termios::Termios {
    let stdin_fd = 0;
    let mut termios = Termios::from_fd(stdin_fd).unwrap();
    let original_config = termios.clone();

    termios.c_lflag &= !(ICANON | ECHO);
    tcsetattr(stdin_fd, TCSANOW, &termios).unwrap();

    original_config
}
