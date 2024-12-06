use postscript_interpreter::{parse, tokenize};
use std::env::args;

fn main() {
    let argv: Vec<String> = args().collect();
    let input = argv[1..].join(" ");
    println!("{}", input);

    let tokens = tokenize(input.as_str()).unwrap();
    let output = parse(tokens).unwrap();
    println!("{}", output);
}
