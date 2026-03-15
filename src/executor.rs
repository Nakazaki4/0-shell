use nix::{libc::fork, sys::wait::waitpid, unistd::ForkResult};

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
        } => {
            execute(*command)?;
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => {
                    let _ = waitpid(child, None);
                    Ok(())
                }
                Ok(ForkResult::Child) => {

                }
            }
        }
        AstNode::Pipe { left, right } => {
            execute(*left)?;
            execute(*right)
        }
    }
}

fn execute_simple_command(command: &String, args: &Vec<String>) -> Result<(), String> {}
