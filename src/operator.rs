use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum OperatorObject {
    Flush,
    Print,
    Quit,
}

impl fmt::Display for OperatorObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperatorObject::Flush => "flush".fmt(f),
            OperatorObject::Print => "print".fmt(f),
            OperatorObject::Quit => "quit".fmt(f),
        }
    }
}
