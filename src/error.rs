use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorKind {
    Unknown,
    UnexpectedEof,
    UnterminatedString,
    UnterminatedArray,
    UnterminatedDictionary,
    MissingValue,
    Syntax,
}

impl From<ErrorKind> for &'static str {
    fn from(value: ErrorKind) -> Self {
        match value {
            ErrorKind::Unknown => "ERR_UNKNOWN",
            ErrorKind::UnexpectedEof => "ERR_UNEXPECTED_EOF",
            ErrorKind::UnterminatedString => "ERR_UNTERMINATED_STR",
            ErrorKind::UnterminatedArray => "ERR_UNTERMINATED_ARRAY",
            ErrorKind::UnterminatedDictionary => "ERR_UNTERMINATED_DICT",
            ErrorKind::MissingValue => "ERR_MISSING_VALUE",
            ErrorKind::Syntax => "ERR_SYNTAX",
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
