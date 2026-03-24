use std::os::fd::AsRawFd;

use nix::fcntl::{OFlag, open};
use nix::libc::dup2;
use nix::sys::stat::Mode;
use nix::{
    sys::wait::waitpid,
    unistd::{ForkResult, close, fork},
};

use crate::builtins::cat::concatenate;
use crate::builtins::cd::change_directory;
use crate::builtins::cp::copy;
use crate::builtins::echo::echo;
use crate::builtins::exit::exit;
use crate::builtins::ls::list;
use crate::builtins::mkdir::make_directory;
use crate::builtins::mv::movee;
use crate::builtins::pwd::print_working_dir;
use crate::builtins::rm::remove;
use crate::parser::{AstNode, Direction};

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
        } => {
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => {
                    let _ = waitpid(child, None);
                    Ok(())
                }
                Ok(ForkResult::Child) => {
                    let (oflags, mode, target_fd) = match direction {
                        Direction::Out => (
                            OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
                            Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
                            1,
                        ),
                        Direction::In => (OFlag::O_RDONLY, Mode::empty(), 0),
                    };

                    let file_fd = open(file.as_str(), oflags, mode)
                        .map_err(|e| format!("Could not open file: {}", e))
                        .unwrap();

                    // file_fd becomes the new out/in instead of the stdout/stdin (changing target_fd with file_fd)
                    if unsafe { dup2(file_fd.as_raw_fd(), target_fd) } < 0 {
                        eprintln!("Failed to redirect stdout");
                        std::process::exit(1);
                    }
                    close(file_fd).expect("Failed to close file descriptor");

                    if let Err(e) = execute(*command) {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }

                    std::process::exit(0);
                }
                Err(e) => Err(format!("Fork failed: {}", e)),
            }
        }
        AstNode::Pipe { left, right } => {
            execute(*left)?;
            execute(*right)
        }
    }
}

fn execute_simple_command(command: &String, args: &Vec<String>) -> Result<(), String> {
    match command.as_str() {
        "cat" => concatenate(args),
        _ => {
            eprintln!("{}: Command not found", command);
            Ok(())
        } // "cd" => change_directory(),
          // "cp" => copy(),
          // "echo" => echo(),
          // "ls" => list(),
          // "mkdir" => make_directory(),
          // "exit" => exit(),
          // "mv" => movee(),
          // "pwd" => print_working_dir(),
          // "rm" => remove(),
    }
}
