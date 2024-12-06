use postscript_interpreter::{parse, token::Token, tokenize};
use std::{io, io::Write};

fn main() {
    let mut input = String::new();
    let _ = io::stdin()
        .read_line(&mut input)
        .expect("could not read input");

    let tokens = tokenize(input.as_str()).unwrap();
    let output = match parse(tokens).expect("failed to parse input") {
        Token::Integer(n) => n.to_string(),
        Token::Real(r) => r.to_string(),
        _ => String::from("Unknown"),
    } + "\n";
    io::stdout()
        .write_all(output.as_bytes())
        .expect("could not write output");
}
