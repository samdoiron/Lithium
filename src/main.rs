use std::str;

struct Message {
    name: String,
    arguments: Vec<(String, Expression)>,
}

struct Expression {
    target: String,
    messages: Vec<Message>,
}

enum Token {
    Identifier(String),
    Number(String),
    ParamName(String),
    Semicolon,
    Then,
}

fn tokenize(code: String) -> Vec<Token> {
    let chars = code.chars().into_iter().peekable();
    while let Some(c) = chars.next() {}
}

fn get_long_token(prev: char, chars: Chars) -> Token {}

fn main() {
    let it = "hello world".to_string();
    let mut chars = it.chars().into_iter().peekable();
    println!("Tokens: {:?}", chars.peek());
}
