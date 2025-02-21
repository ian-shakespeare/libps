use crate::{Error, ErrorKind, Interpreter};

#[derive(Clone, Debug)]
pub enum Object {
    Integer(i32),
    Real(f64),
    Boolean(bool),
    Array(usize),
    PackedArray(usize),
    Procedure(usize),
    String(usize),
    Dictionary(usize),
    Name(String),
    Operator(fn(&mut Interpreter) -> crate::Result<()>),
    File,
    Mark,
    Null,
    Save,
    FontId, // TODO: Figure out what these things are
    GState,
}

impl From<Object> for String {
    fn from(value: Object) -> Self {
        match value {
            Object::Integer(value) => value.to_string(),
            Object::Real(value) => value.to_string(),
            Object::Boolean(value) => value.to_string(),
            Object::Name(value) => value,
            Object::Array(_) => "array".to_string(),
            Object::PackedArray(_) => "packedarray".to_string(),
            Object::Mark => "mark".to_string(),
            Object::Null => "null".to_string(),
            _ => "".to_string(),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Integer(value) => match other {
                Self::Integer(other_value) => value == other_value,
                Self::Real(other_value) => f64::from(*value) == *other_value,
                _ => false,
            },
            Self::Real(value) => match other {
                Self::Integer(other_value) => *value == f64::from(*other_value),
                Self::Real(other_value) => value == other_value,
                _ => false,
            },
            Self::Boolean(value) => match other {
                Self::Boolean(other_value) => value == other_value,
                _ => false,
            },
            Self::Array(value) => match other {
                Self::Array(other_value) => value == other_value,
                _ => false,
            },
            Self::PackedArray(value) => match other {
                Self::PackedArray(other_value) => value == other_value,
                _ => false,
            },
            Self::String(value) => match other {
                Self::String(other_value) => value == other_value,
                _ => false,
            },
            Self::Name(value) => match other {
                Self::Name(other_value) => value == other_value,
                _ => false,
            },
            Self::Null => matches!(other, Self::Null),
            _ => false,
        }
    }
}

impl Eq for Object {}

impl Object {
    pub fn name(&self) -> &str {
        match self {
            Self::Integer(..) => "integer",
            Self::Real(..) => "real",
            Self::Boolean(..) => "boolean",
            Self::Name(..) => "name",
            Self::PackedArray(..) => "packedarray",
            Self::Array(..) => "array",
            Self::Procedure(..) => "procedure",
            Self::String(..) => "string",
            Self::Dictionary(..) => "dictionary",
            Self::Operator(..) => "operator",
            Self::File => "file",
            Self::Mark => "mark",
            Self::Null => "null",
            Self::Save => "save",
            Self::FontId => "fontid",
            Self::GState => "gstate",
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Integer(..))
    }

    pub fn is_real(&self) -> bool {
        matches!(self, Self::Real(..))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }

    pub fn is_mark(&self) -> bool {
        matches!(self, Self::Mark)
    }

    pub fn is_procedure(&self) -> bool {
        matches!(self, Self::Procedure(..))
    }

    pub fn into_int(&self) -> crate::Result<i32> {
        match self {
            Self::Integer(i) => Ok(*i),
            _ => Err(Error::new(
                ErrorKind::TypeCheck,
                format!("expected integer, received {}", self.name()),
            )),
        }
    }

    pub fn into_real(&self) -> crate::Result<f64> {
        match self {
            Self::Integer(i) => Ok(f64::from(*i)),
            Self::Real(r) => Ok(*r),
            _ => Err(Error::new(
                ErrorKind::TypeCheck,
                format!("expected real, received {}", self.name()),
            )),
        }
    }

    pub fn into_usize(&self) -> crate::Result<usize> {
        match self {
            Self::Integer(i) => match (*i).try_into() {
                Ok(u) => Ok(u),
                Err(_) => Err(Error::from(ErrorKind::RangeCheck)),
            },
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected usize")),
        }
    }
}
