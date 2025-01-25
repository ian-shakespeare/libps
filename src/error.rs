use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorKind {
    IoError,
    LimitCheck,
    RangeCheck,
    StackUnderflow,
    Syntax,
    TypeCheck,
    Undefined,
    UndefinedResult,
    UnmatchedMark,
    Unregistered,
}

impl From<ErrorKind> for &'static str {
    fn from(value: ErrorKind) -> Self {
        match value {
            ErrorKind::IoError => "ioerror",
            ErrorKind::LimitCheck => "limitcheck",
            ErrorKind::RangeCheck => "rangecheck",
            ErrorKind::StackUnderflow => "stackunderflow",
            ErrorKind::Syntax => "syntaxerror",
            ErrorKind::TypeCheck => "typecheck",
            ErrorKind::Undefined => "undefined",
            ErrorKind::UndefinedResult => "undefinedresult",
            ErrorKind::UnmatchedMark => "unmatchedmark",
            ErrorKind::Unregistered => "unregistered",
        }
    }
}

#[derive(Debug)]
pub struct Error {
    error: Box<dyn std::error::Error + Send + Sync>,
    kind: ErrorKind,
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            error: error.into(),
            kind,
        }
    }

    pub fn from(kind: ErrorKind) -> Self {
        Self {
            error: "".into(),
            kind,
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: &str = self.kind().into();
        let message: String = self.error.to_string();
        write!(f, "{}: {}", name, message)
    }
}

impl std::error::Error for Error {}
