use errors::ErrorKind;
use scanner::Scanner;

pub mod errors;
mod scanner;
pub mod token;
pub mod traits;

pub fn scan(input: &'static str) {
    let mut scanner = Scanner::new(input);
    loop {
        match scanner.read_token() {
            Ok(t) => println!("{:?}", t),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => println!("{:?}", e),
        }
    }
}
