use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::{
    sys::wait::waitpid,
    unistd::{ForkResult, close, dup2, fork}
};

use crate::parser::AstNode;

pub fn execute(node: AstNode) -> Result<(), String> {
    match node {
        AstNode::SimpleCommand { name, args } => execute_simple_command(&name, &args),
        AstNode::Sequence { left, right } => {
            execute(*left)?;
            execute(*right)
        }
        AstNode::Redirect {
            command,
            file,
            direction,
        } => match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                let _ = waitpid(child, None);
                Ok(())
            }
            Ok(ForkResult::Child) => {
                let oflags = OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC;
                let mode = Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH;

                let file_fd = open(file.as_str(), oflags, mode)
                    .map_err(|e| format!("Could not open file: {}", e))
                    .unwrap();

                dup2(file_fd, 1).expect("Failed to redirect stdout");

                close(file_fd).expect("Failed to close file descriptor");

                if let Err(e) = execute(*command) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }

                std::process::exit(0);
            }
            Err(e) => Err(format!("Fork failed: {}", e)),
        },
        AstNode::Pipe { left, right } => {
            execute(*left)?;
            execute(*right)
        }
    }
}

fn execute_simple_command(command: &String, args: &Vec<String>) -> Result<(), String> {}
