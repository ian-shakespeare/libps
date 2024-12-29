#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorKind {
    Unknown,
    UnexpectedEof,
    UnterminatedString,
    Read,
    Syntax,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    cause: Option<Box<dyn std::error::Error>>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { cause: None, kind }
    }

    pub fn with_cause(kind: ErrorKind, cause: Box<dyn std::error::Error>) -> Self {
        Self {
            cause: Some(cause),
            kind,
        }
    }

    pub fn eof() -> Self {
        Self {
            kind: ErrorKind::UnexpectedEof,
            cause: None,
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}
