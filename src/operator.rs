use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum OperatorObject {
    Cvlit,
    Cvx,
    Exec,
    Flush,
    Print,
    Quit,
}

impl fmt::Display for OperatorObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperatorObject::Cvlit => "cvlit".fmt(f),
            OperatorObject::Cvx => "cvx".fmt(f),
            OperatorObject::Exec => "exec".fmt(f),
            OperatorObject::Flush => "flush".fmt(f),
            OperatorObject::Print => "print".fmt(f),
            OperatorObject::Quit => "quit".fmt(f),
        }
    }
}
