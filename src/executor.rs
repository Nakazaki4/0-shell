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
use crate::parser::{AstNode};

pub fn execute(node: AstNode) -> Result<(), String> {
    match node {
        AstNode::SimpleCommand { name, args } => execute_simple_command(&name, &args),
        AstNode::Sequence { left, right } => {
            execute(*left)?;
            execute(*right)
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
