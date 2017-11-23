mod object;
mod parser;
mod eval;

use parser::{tokenize, parse_program};
use eval::Program;

use std::io::{self, Read};

fn main() {
    let mut program = String::new();
    io::stdin().read_to_string(&mut program).unwrap();
    let tokens = tokenize(program);
    let parsed = parse_program(tokens);
    // println!("{:#?}", parsed);
    Program::new().eval(parsed);
}
