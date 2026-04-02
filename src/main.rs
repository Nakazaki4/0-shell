// Lexer -> Parser -> Executor
mod builtins;
mod executor;
mod lexer;
mod parser;

use lexer::tokenize;
use parser::parse;
use std::io::{self, Read, Write};
use std::{env, process};
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
        if let Ok(dir) = env::current_dir() {
            unsafe { env::set_var("PWD", dir) }
        }
        // 2. Fetch the prompt string first
        let prompt = match get_prompt() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("\r\n{}", e);
                break;
            }
        };

        // 3. Print the initial prompt
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        // 4. Pass BOTH the prompt and history to the reader
        let full_input = match read_command(&prompt, &history) {
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

        // 5. Safely handle parser errors without panicking
        let command_tree = match parse(tokenized_command) {
            Ok(tree) => tree,
            Err(e) => {
                eprintln!("{}\r", e); // Fixed double space bug
                continue;
            }
        };

        // 6. RAW MODE TOGGLE: Turn raw mode OFF before executing commands like `cat`
        tcsetattr(0, TCSANOW, &original_config).unwrap();

        match execute(command_tree) {
            Ok(_) => {}
            Err(e) => eprintln!("{}", e), // Normal mode, so normal println is fine
        }

        // 7. Turn raw mode BACK ON for the next prompt loop
        switch_to_raw_mode();
    }

    tcsetattr(0, TCSANOW, &original_config).unwrap();
}

fn get_prompt() -> Result<String, String> {
    let display_path = match env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(_) => match env::var("PWD") {
            Ok(path) => path,
            Err(_) => {
                return Err("cannot get the current working directory".to_string());
            }
        },
    };

    Ok(format!("{} $ ", display_path))
}

fn read_command(base_prompt: &str, history: &[String]) -> Option<String> {
    let mut full_input = String::new();
    let mut buffer = [0; 1];

    let mut current_prompt = base_prompt.to_string();
    let mut history_index = history.len();

    loop {
        io::stdin().read_exact(&mut buffer).unwrap();
        let byte = buffer[0];

        if byte == b'\r' || byte == b'\n' {
            let trimmed = full_input.trim_end();

            if trimmed.ends_with('\\') {
                let slash_pos = full_input.rfind('\\').unwrap();
                full_input.truncate(slash_pos);

                current_prompt = "> ".to_string();
                print!("\r\n{}", current_prompt);
                io::stdout().flush().unwrap();
                continue;
            // --- MULTI-LINE QUOTE LOGIC ---
            } else if has_unclosed_double_quote(&full_input) {
                full_input.push('\n');
                current_prompt = "dquote> ".to_string();
                print!("\r\n{}", current_prompt);
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
                    if history_index > 0 {
                        history_index -= 1;
                        full_input = history[history_index].clone();

                        print!("\x1b[2K\r{}{}", current_prompt, full_input);
                        io::stdout().flush().unwrap();
                    }
                } else if buffer[0] == DOWN {
                    if history_index < history.len() {
                        history_index += 1;

                        if history_index == history.len() {
                            full_input.clear(); // Reached the bottom, clear the line
                        } else {
                            full_input = history[history_index].clone();
                        }

                        print!("\x1b[2K\r{}{}", current_prompt, full_input);
                        io::stdout().flush().unwrap();
                    }
                }
            }
        } else if byte == TAB {
            // TODO: auto completion
        } else if byte >= 32 {
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

fn has_unclosed_double_quote(input: &str) -> bool {
    let mut in_quote = false;
    let mut escaped = false;

    for c in input.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == '"' {
            in_quote = !in_quote;
        }
    }
    in_quote
}
