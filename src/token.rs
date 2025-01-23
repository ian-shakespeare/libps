#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenKind {
    Integer,
    Real,
    StringLiteral,
    StringHex,
    StringBase85,
    Name,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    value: String,
    kind: TokenKind,
}

impl Token {
    pub fn new(kind: TokenKind, value: String) -> Self {
        Self { value, kind }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
