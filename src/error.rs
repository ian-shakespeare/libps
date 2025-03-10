use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorKind {
    DictStackUnderflow,
    InvalidAccess,
    IoError,
    LimitCheck,
    RangeCheck,
    StackUnderflow,
    SyntaxError,
    TypeCheck,
    Undefined,
    UndefinedResult,
    UnmatchedMark,
    Unregistered,
    VmError,
}

impl From<ErrorKind> for &'static str {
    fn from(value: ErrorKind) -> Self {
        match value {
            ErrorKind::DictStackUnderflow => "dictstackunderflow",
            ErrorKind::InvalidAccess => "invalidaccess",
            ErrorKind::IoError => "ioerror",
            ErrorKind::LimitCheck => "limitcheck",
            ErrorKind::RangeCheck => "rangecheck",
            ErrorKind::StackUnderflow => "stackunderflow",
            ErrorKind::SyntaxError => "syntaxerror",
            ErrorKind::TypeCheck => "typecheck",
            ErrorKind::Undefined => "undefined",
            ErrorKind::UndefinedResult => "undefinedresult",
            ErrorKind::UnmatchedMark => "unmatchedmark",
            ErrorKind::Unregistered => "unregistered",
            ErrorKind::VmError => "vmerror",
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

        if message.is_empty() {
            write!(f, "{name}")
        } else {
            write!(f, "{name}: {message}")
        }
    }
}

impl std::error::Error for Error {}
