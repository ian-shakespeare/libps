use std::{io, process};

use libps::eval;

fn main() {
    let input = io::stdin();
    let mut buf = String::new();
    if input.read_line(&mut buf).is_err() {
        panic!("Failed to read input.");
    }

    if let Err(e) = eval(&buf) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
