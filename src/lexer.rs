pub fn tokenize(command: &str) -> Vec<String> {
    let mut tokens_vec: Vec<String> = Vec::new();
    let mut token = String::new();
    let mut is_inside_quote = false;
    let mut is_escaped = false;

    for c in command.chars() {
        if is_escaped {
            token.push(c);
            is_escaped = false;
            continue;
        }

        if c == '\\' {
            is_escaped = true;
            continue;
        }
        if c == '\"' {
            is_inside_quote = !is_inside_quote;
            continue;
        }
        if is_inside_quote {
            token.push(c);
            continue;
        }
        if c.is_whitespace() {
            if !token.is_empty() {
                tokens_vec.push(token.clone());
                token.clear();
            }
            continue;
        }
        if c == '|' || c == '>' || c == '<' || c == ';' {
            if !token.is_empty() {
                tokens_vec.push(token.clone());
                token.clear();
            }
            tokens_vec.push(c.to_string());
            continue;
        }
        token.push(c);
    }
    if !token.is_empty() {
        tokens_vec.push(token);
    }
    tokens_vec
}
