#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Integer(i32),
    Real(f64),
    String(String),
    Name(String),
}
