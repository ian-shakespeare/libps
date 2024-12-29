#[derive(Debug, PartialEq)]
pub enum Token {
    Unknown,
    Integer(i32),
    Real(f64),
    String(String),
    Name(String),
}

/*
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Integer(self_value) => match other {
                Self::Integer(other_value) => self_value == other_value,
                _ => false,
            },
            Self::Real(self_value) => match other {
                Self::Real(other_value) => self_value == other_value,
                _ => false,
            },
            Self::String(self_value) => match other {
                Self::String(other_value) => self_value == other_value,
                _ => false,
            },
            Self::Name(self_value) => match other {
                Self::Name(other_value) => self_value == other_value,
                _ => false,
            },
            _ => false,
        }
    }
}
*/
