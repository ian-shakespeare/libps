#[derive(Debug, PartialEq)]
pub enum Token {
    Unknown,
    Integer(i32),
    Real(f64),
    String(String),
    Name(String),
}
