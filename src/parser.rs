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
    if let Some(pos) = tokens
        .iter()
        .position(|t| t == ";" || t == "|" || t == ">" || t == "<")
    {
        let splitted = tokens.split_at(pos);
        let left_tokens = splitted.0;
        let right_tokens = splitted.1;

        return match tokens[pos].as_str() {
            ";" => Ok(AstNode::Sequence {
                left: Box::new(parse(left_tokens.to_vec())?),
                right: Box::new(parse(right_tokens.to_vec())?),
            }),
            ">" => Ok(AstNode::Redirect {
                command: Box::new(parse(left_tokens.to_vec())?),
                file: right_tokens.concat(),
                direction: Direction::Out,
            }),
            "<" => Ok(AstNode::Redirect {
                command: Box::new(parse(right_tokens.to_vec())?),
                file: left_tokens.concat(),
                direction: Direction::In,
            }),
            "|" => Ok(AstNode::Pipe {
                left: Box::new(parse(left_tokens.to_vec())?),
                right: Box::new(parse(right_tokens.to_vec())?),
            }),
            _ => unreachable!(),
        };
    }

    // in simple command case
    let splitted = tokens.split_at(1);
    let name = splitted.0.concat();
    let args = splitted.1.to_vec();

    Ok(AstNode::SimpleCommand { name, args })
}
