use encoding::decode_ascii85;
pub use error::{Error, ErrorKind};
use scanner::Scanner;
use std::io;

mod encoding;
mod error;
mod scanner;
pub mod token;
pub mod traits;

pub type Result<T> = std::result::Result<T, crate::Error>;

pub trait Peek {
    fn peek(&self, buf: &mut [u8]) -> io::Result<usize>;
}

pub trait PeekRead: Peek + io::Read {}
impl<T: Peek + io::Read> PeekRead for T {}

pub fn scan(input: &'static str) {
    let mut scanner = Scanner::new(input);
    loop {
        match scanner.read_token() {
            Result::Ok(t) => println!("{:?}", t),
            Result::Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Result::Err(e) => println!("{:?}", e),
        }
    }
}

pub fn encode() {
    let out = decode_ascii85("FD,B0+DGm>@3B#fF(I<g+EMXFBl7P").unwrap();
    println!("{}", out);
}
