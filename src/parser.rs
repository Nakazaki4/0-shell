#[derive(Debug)]
pub enum Direction {
    In,  // <
    Out, // >
}

#[derive(Debug)]
pub enum AstNode {
    SimpleCommand {
        name: String,
        args: Vec<String>,
    },

    // ;
    Sequence {
        left: Box<AstNode>,
        right: Box<AstNode>,
    },

    Pipe {
        left: Box<AstNode>,
        right: Box<AstNode>,
    },

    // > or <
    Redirect {
        command: Box<AstNode>,
        file: String,
        direction: Direction,
    },
}

pub fn parse(tokens: Vec<String>) -> Result<AstNode, String> {
    if tokens.is_empty() {
        return Err("bash: syntax error near unexpected token".to_string());
    }

    if let Some(pos) = tokens
        .iter()
        .position(|t| t == ";")
    {
        let left_tokens = &tokens[..pos];
        let right_tokens = &tokens[pos + 1..];
        return match tokens[pos].as_str() {
            ";" => Ok(AstNode::Sequence {
                left: Box::new(parse(left_tokens.to_vec())?),
                right: Box::new(parse(right_tokens.to_vec())?),
            }),
            _ => unreachable!(),
        };
    }

    let name = tokens[0].clone();
    let args = tokens[1..].to_vec();

    Ok(AstNode::SimpleCommand { name, args })
}
