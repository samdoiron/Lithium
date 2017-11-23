use std::iter::Peekable;
use std::vec;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Message {
    pub name: String,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: String,
    pub value: Expression
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>
}

#[derive(Debug, Clone)]
pub enum Target {
    Number(String),
    Identifier(String),

    // NOTE: Heap allocation :(
    Expression(Box<Expression>)
}

#[derive(Debug, Clone)]
pub struct Send {
    pub target: Target,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Send(Send),
    Number(String),

    // NOTE: Heap allocation :(
    Lambda(Box<Block>),
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub target: String,
    pub value: Expression
}

#[derive(Debug, Clone)]
pub enum Statement{
    Expression(Expression),
    Definition(Definition)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Identifier(String),
    Number(String),
    ParamName(String),
    NextStatement,
    Def,
    Then,
    OpenParen,
    CloseParen,
    OpenLambda,
    CloseLambda
}

type Tokens = Peekable<vec::IntoIter<Token>>;

pub fn parse_program(tokens: Vec<Token>) -> Block {
    let mut token_iter = tokens.into_iter().peekable();
    parse_block(&mut token_iter)
}

fn parse_block(tokens: &mut Tokens) -> Block {
    let mut block = Block{statements: Vec::new()};

    while tokens.peek().is_some() {
        block.statements.push(parse_statement(tokens));
        match tokens.peek() {
            Some(&Token::NextStatement) => { tokens.next(); },
            Some(&Token::CloseLambda) => break,
            None => (),
            _ => panic!("Unknown remaining tokens after parsing statement")
        }
    }

    return block;
}

fn parse_statement(tokens: &mut Tokens) -> Statement {
    // <subject|identifier> <message|identifier>
    match tokens.peek().cloned() {
        Some(Token::Def) => Statement::Definition(parse_definition(tokens)),
        Some(_) => Statement::Expression(parse_expression(tokens)),
        None => panic!("Ran out of tokens in statement D:")
    }
}

fn parse_definition(tokens: &mut Tokens) -> Definition {
    // def <identifier> <expression>
    match (tokens.next(), tokens.next()) {
        (Some(Token::Def), Some(Token::Identifier(identifier))) => {
            Definition {
                target: identifier,
                value: parse_expression(tokens)
            }
        },
        _ => panic!("Uh oh, malformed definition")
    }
}

fn parse_expression(tokens: &mut Tokens) -> Expression {
    match (tokens.next(), tokens.peek()) {
        (Some(Token::OpenLambda), _) => {
            let lambda = Expression::Lambda(Box::new(parse_block(tokens)));
            match tokens.next() {
                Some(Token::CloseLambda) => (),
                _ => panic!("Expected lambda to end with a closing bracket")
            }
            lambda
        },
        // (myCar start) println
        (Some(Token::OpenParen), _) => {
            let subject = parse_expression(tokens);
            tokens.next(); // Remove the remaining ')'
            match tokens.peek() {
                // There is a message being sent to the result
                // DUPE from below. Extract this parsing logic?
                Some(&Token::Identifier(_)) => {
                    let message = match tokens.next() {
                        Some(Token::Identifier(message)) => message,
                        _ => unreachable!()
                    };
                    Expression::Send(Send{
                        target: Target::Expression(Box::new(subject)),
                        messages: vec![
                            Message{name: message, arguments: parse_send_arguments(tokens)}
                        ]
                    })
                },
                _ => subject
            }
        },
        // myCar start
        (Some(Token::Identifier(subject)), Some(&Token::Identifier(_))) => {
            let message = match tokens.next() {
                Some(Token::Identifier(message)) => message,
                _ => unreachable!()
            };
            Expression::Send(Send{
                target: Target::Identifier(subject),
                messages: vec![
                    Message{name: message, arguments: parse_send_arguments(tokens)}
                ]
            })
        },
        // 123 println
        (Some(Token::Number(num)), Some(&Token::Identifier(_))) => {
            let message = match tokens.next() {
                Some(Token::Identifier(message)) => message,
                _ => unreachable!()
            };
            Expression::Send(Send{
                target: Target::Number(num),
                messages: vec![
                    Message{name: message, arguments: parse_send_arguments(tokens)}
                ]
            })
        },
        (Some(Token::Number(num)), _) => Expression::Number(num),
        (None, None) => panic!("Uh oh, ran out of tokens in expression"),
        _ => panic!("Unhandled tokens")
    }
}

fn parse_send_arguments(tokens: &mut Tokens) -> Vec<Argument> {
    let mut params = Vec::new();
    while let Some(Token::ParamName(name)) = tokens.peek().cloned() {
        tokens.next();
        params.push(Argument{
            name: name,
            value: parse_expression(tokens)
        });
    }
    return params
}

pub fn tokenize(code: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = code.chars().into_iter().peekable();
    while let Some(c) = chars.next() {
        if c != '\n' && c.is_whitespace() { continue }
        tokens.push(match c {
            '\n' => Token::NextStatement,
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            '[' => Token::OpenLambda,
            ']' => Token::CloseLambda,
            _ => get_long_token(c.clone(), &mut chars)
        });
    }
    return tokens;
}

fn get_long_token(prev: char, chars: &mut Peekable<Chars>) -> Token {
    if prev.is_alphabetic() {
        let mut name = String::new();
        name.push(prev);
        while let Some(c) = chars.peek().cloned() {
            if !c.is_alphabetic() {
                if c == ':' {
                    chars.next();
                    return Token::ParamName(name);
                }
                break;
            }
            name.push(chars.next().unwrap())
        }
        // Keywords
        match name {
            ref s if s == "def" => Token::Def,
            ref s if s == "then" => Token::Then,
            _ => Token::Identifier(name)
        }
    } else if prev.is_numeric() {
        let mut number = String::new();
        number.push(prev);
        while let Some(c) = chars.peek().cloned() {
            if !c.is_numeric() { break }
            number.push(chars.next().unwrap())
        }
        Token::Number(number)
    } else {
        unreachable!()
    }
}