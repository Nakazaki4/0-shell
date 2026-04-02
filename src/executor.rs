use std::os::fd::AsRawFd;

use nix::fcntl::{OFlag, open};
use nix::libc::dup2;
use nix::sys::stat::Mode;
use nix::unistd::pipe;
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
            let (read_fd, write_fd) = pipe().map_err(|e| format!("Pipe failed: {}", e))?;
            match unsafe { fork() } {
                Ok(ForkResult::Child) => {
                    let _ = close(read_fd.as_raw_fd());

                    if unsafe { dup2(write_fd.as_raw_fd(), 1) } < 0 {
                        eprintln!("Failed to redirect stdout to pipe");
                        std::process::exit(1);
                    }

                    let _ = close(write_fd.as_raw_fd());

                    if let Err(e) = execute(*left) {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                    std::process::exit(0);
                }
                Ok(ForkResult::Parent { child: left_child }) => match unsafe { fork() } {
                    Ok(ForkResult::Child) => {
                        let _ = close(write_fd.as_raw_fd());

                        if unsafe { dup2(read_fd.as_raw_fd(), 0) } < 0 {
                            eprintln!("Failed to redirect stdin from pipe");
                            std::process::exit(1);
                        }

                        let _ = close(read_fd.as_raw_fd());

                        if let Err(e) = execute(*right) {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                        std::process::exit(0);
                    }
                    Ok(ForkResult::Parent { child: right_child }) => {
                        let _ = close(read_fd.as_raw_fd());
                        let _ = close(write_fd.as_raw_fd());

                        let _ = waitpid(left_child, None);
                        let _ = waitpid(right_child, None);

                        Ok(())
                    }
                    Err(e) => Err(format!("Fork failed for right side of pipe: {}", e)),
                },
                Err(e) => Err(format!("Fork failed for left side of pipe: {}", e)),
            }
        }
    }
}

fn execute_simple_command(command: &String, args: &Vec<String>) -> Result<(), String> {
    match command.as_str() {
        "cat" => concatenate(args),
        "cd" => change_directory(args),
        "cp" => copy(args),
        "echo" => echo(args),
        "exit" => exit(args),
        "ls" => Ok(list(args)),
        "mkdir" => make_directory(args),
        "mv" => movee(args),
        "pwd" => print_working_dir(args),
        "rm" => Ok(remove(args)),
        _ => {
            eprintln!("{}: Command not found", command);
            Ok(())
        }
    }
}
