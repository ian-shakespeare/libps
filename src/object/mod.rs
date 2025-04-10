use crate::{context::Context, Error, ErrorKind};
pub use array_object::ArrayObject;
pub use dictionary_object::DictionaryObject;
pub use name_object::NameObject;
pub use operator_object::OperatorObject;
pub use string_object::StringObject;

mod array_object;
mod dictionary_object;
mod name_object;
mod operator_object;
mod string_object;

#[derive(Clone, Debug)]
pub enum Object {
    /* Simple */
    Boolean(bool),
    FontId,
    Integer(i32),
    Mark,
    Name(NameObject),
    Null,
    Operator(OperatorObject),
    Real(f64),

    /* Composite */
    Array(usize),
    Dictionary(usize),
    File(usize),
    GState(usize),
    Save(usize),
    String(usize),
}

impl Object {
    pub fn into_index(self) -> crate::Result<usize> {
        match self {
            Object::Array(idx)
            | Object::Dictionary(idx)
            | Object::File(idx)
            | Object::GState(idx)
            | Object::Save(idx)
            | Object::String(idx) => Ok(idx),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected composite")),
        }
    }

    pub fn into_int(self) -> crate::Result<i32> {
        match self {
            Self::Integer(i) => Ok(i),
            _ => {
                let received: &str = self.into();
                Err(Error::new(
                    ErrorKind::TypeCheck,
                    format!("expected int, received {received}"),
                ))
            },
        }
    }

    pub fn into_name(self) -> crate::Result<NameObject> {
        match self {
            Self::Name(n) => Ok(n),
            _ => {
                let received: &str = self.into();
                Err(Error::new(
                    ErrorKind::TypeCheck,
                    format!("expected name, received {received}"),
                ))
            },
        }
    }

    pub fn into_real(self) -> crate::Result<f64> {
        match self {
            Self::Integer(i) => Ok(f64::from(i)),
            Self::Real(r) => Ok(r),
            _ => {
                let received: &str = self.into();
                Err(Error::new(
                    ErrorKind::TypeCheck,
                    format!("expected real, received {received}"),
                ))
            },
        }
    }

    pub fn into_usize(self) -> crate::Result<usize> {
        let u: usize = self
            .into_int()?
            .try_into()
            .or(Err(Error::from(ErrorKind::RangeCheck)))?;

        Ok(u)
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(..))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Integer(..))
    }

    pub fn is_mark(&self) -> bool {
        matches!(self, Self::Mark)
    }

    pub fn is_name(&self) -> bool {
        matches!(self, Self::Name(..))
    }

    pub fn is_numeric(&self) -> bool {
        self.is_int() || self.is_real()
    }

    pub fn is_real(&self) -> bool {
        matches!(self, Self::Real(..))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }

    pub fn mode(&self, ctx: &Context) -> crate::Result<Mode> {
        match self {
            Self::Name(NameObject { mode, .. }) => Ok(*mode),
            Self::Array(idx)
            | Self::Dictionary(idx)
            | Self::File(idx)
            | Self::GState(idx)
            | Self::Save(idx) => {
                let comp = ctx.mem().get(*idx).ok_or(Error::from(ErrorKind::VmError))?;

                match comp {
                    Composite::Array(ArrayObject { mode, .. }) => Ok(*mode),
                    Composite::Dictionary(DictionaryObject { mode, .. }) => Ok(*mode),
                    _ => Ok(Mode::default()),
                }
            },
            _ => Ok(Mode::default()),
        }
    }
}

impl From<Object> for &'static str {
    fn from(value: Object) -> Self {
        match value {
            Object::Boolean(_) => "booleantype",
            Object::FontId => "fonttype",
            Object::Integer(_) => "integertype",
            Object::Mark => "marktype",
            Object::Name(_) => "nametype",
            Object::Null => "nulltype",
            Object::Operator(_) => "operatortype",
            Object::Real(_) => "realtype",
            Object::Array(_) => "arraytype",
            Object::Dictionary(_) => "dicttype",
            Object::File(_) => "filetype",
            Object::GState(_) => "gstatetype",
            Object::Save(_) => "savetype",
            Object::String(_) => "stringtype",
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if self.is_numeric() && other.is_numeric() {
            let Ok(lhs) = self.clone().into_real() else {
                return false;
            };
            let Ok(rhs) = other.clone().into_real() else {
                return false;
            };

            return lhs == rhs;
        }

        match (self, other) {
            (Object::Boolean(lhs), Object::Boolean(rhs)) => lhs == rhs,
            (Object::String(lhs), Object::String(rhs)) => lhs == rhs,
            (Object::Name(lhs), Object::Name(rhs)) => lhs == rhs,
            (Object::Array(lhs), Object::Array(rhs)) => lhs == rhs,
            (Object::Dictionary(lhs), Object::Dictionary(rhs)) => lhs == rhs,
            (Object::Operator(lhs), Object::Operator(rhs)) => lhs == rhs,
            (Object::Mark, Object::Mark) => true,
            (Object::Null, Object::Null) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Mode {
    #[default]
    Executable,
    Literal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Access {
    #[default]
    Unlimited,
    ReadOnly,
    ExecuteOnly,
    None,
}

impl Access {
    pub fn is_writeable(&self) -> bool {
        *self == Self::Unlimited
    }

    pub fn is_readable(&self) -> bool {
        self.is_writeable() || *self == Self::ReadOnly
    }

    pub fn is_executable(&self) -> bool {
        self.is_readable() || *self == Self::ExecuteOnly
    }

    pub fn is_exec_only(&self) -> bool {
        *self == Self::ExecuteOnly
    }
}

#[derive(Clone, Debug)]
pub enum Composite {
    Array(ArrayObject),
    Dictionary(DictionaryObject),
    String(StringObject),
}

impl From<ArrayObject> for Composite {
    fn from(value: ArrayObject) -> Self {
        Self::Array(value)
    }
}

impl From<DictionaryObject> for Composite {
    fn from(value: DictionaryObject) -> Self {
        Self::Dictionary(value)
    }
}

impl From<StringObject> for Composite {
    fn from(value: StringObject) -> Self {
        Self::String(value)
    }
}
