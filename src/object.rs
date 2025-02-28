use crate::{
    composite::{Composite, Mode},
    Error, ErrorKind, Interpreter,
};

#[derive(Clone, Debug)]
pub struct Name {
    pub mode: Mode,
    value: String,
}

impl Name {
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl<S> From<S> for Name
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self {
            value: value.into(),
            mode: Mode::Executable,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Object {
    Integer(i32),
    Real(f64),
    Boolean(bool),
    Array(Composite),
    PackedArray(Composite),
    String(Composite),
    Dictionary(Composite),
    Name(Name),
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
            Object::Name(Name { value, .. }) => value,
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
            Self::Name(Name { value, .. }) => match other {
                Self::Name(Name {
                    value: other_value, ..
                }) => value == other_value,
                _ => false,
            },
            Self::Null => matches!(other, Self::Null),
            Self::Array(Composite { key, .. })
            | Self::PackedArray(Composite { key, .. })
            | Self::String(Composite { key, .. })
            | Self::Dictionary(Composite { key, .. }) => match other {
                Self::Array(Composite { key: other_key, .. })
                | Self::PackedArray(Composite { key: other_key, .. })
                | Self::String(Composite { key: other_key, .. })
                | Self::Dictionary(Composite { key: other_key, .. }) => key == other_key,
                _ => false,
            },
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

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(..))
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

    pub fn into_composite(&self) -> crate::Result<Composite> {
        match self {
            Self::Array(c) | Self::PackedArray(c) | Self::String(c) | Self::Dictionary(c) => {
                Ok(c.clone())
            },
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected composite")),
        }
    }
}
