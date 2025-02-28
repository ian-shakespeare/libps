use std::collections::HashMap;

use crate::{context::Context, Error, ErrorKind};

#[derive(Clone)]
pub enum Object {
    /* Simple */
    Boolean(bool),
    FontId,
    Integer(i32),
    Mark,
    Name(NameObject),
    Null,
    Operator(fn(&mut Context) -> crate::Result<()>),
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
    pub fn mode(&self, ctx: &Context) -> Option<Mode> {
        match self {
            Self::Name(NameObject { mode, .. }) => Some(*mode),
            Self::Boolean(_)
            | Self::FontId
            | Self::Integer(_)
            | Self::Mark
            | Self::Null
            | Self::Operator(_)
            | Self::Real(_)
            | Self::String(_) => Some(Mode::Literal),
            Self::Array(idx)
            | Self::Dictionary(idx)
            | Self::File(idx)
            | Self::GState(idx)
            | Self::Save(idx) => {
                let comp = ctx.mem().get(*idx)?;

                match comp {
                    Composite::Array(ArrayObject { mode, .. }) => Some(*mode),
                    Composite::Dictionary(DictionaryObject { mode, .. }) => Some(*mode),
                    _ => None,
                }
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Literal,
    Executable,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Access {
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
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct ArrayObject {
    access: Access,
    mode: Mode,
    inner: Vec<Object>,
}

impl ArrayObject {
    pub fn new<V>(value: V, access: Access, mode: Mode) -> Self
    where
        V: Into<Vec<Object>>,
    {
        Self {
            inner: value.into(),
            access,
            mode,
        }
    }

    pub fn access(&self) -> Access {
        self.access
    }
}

impl IntoIterator for ArrayObject {
    type Item = Object;
    type IntoIter = <Vec<Object> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl TryFrom<Composite> for ArrayObject {
    type Error = crate::Error;

    fn try_from(value: Composite) -> Result<Self, Self::Error> {
        match value {
            Composite::Array(a) => Ok(a),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }
}

impl<'a> TryFrom<&'a Composite> for &'a ArrayObject {
    type Error = crate::Error;

    fn try_from(value: &'a Composite) -> Result<Self, Self::Error> {
        match value {
            Composite::Array(a) => Ok(a),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }
}

#[derive(Clone)]
pub struct DictionaryObject {
    access: Access,
    mode: Mode,
    inner: HashMap<String, Object>,
}

impl DictionaryObject {
    pub fn get(&self, key: &str) -> Option<&Object> {
        self.inner.get(key)
    }
}

impl IntoIterator for DictionaryObject {
    type Item = (String, Object);
    type IntoIter = <HashMap<String, Object> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl TryFrom<Composite> for DictionaryObject {
    type Error = crate::Error;

    fn try_from(value: Composite) -> Result<Self, Self::Error> {
        match value {
            Composite::Dictionary(d) => Ok(d),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }
}

impl<'a> TryFrom<&'a Composite> for &'a DictionaryObject {
    type Error = crate::Error;

    fn try_from(value: &'a Composite) -> Result<Self, Self::Error> {
        match value {
            Composite::Dictionary(d) => Ok(d),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }
}

#[derive(Clone)]
pub struct StringObject {
    inner: String,
}

impl From<String> for StringObject {
    fn from(value: String) -> Self {
        Self { inner: value }
    }
}

impl<'a> From<&'a StringObject> for &'a str {
    fn from(value: &'a StringObject) -> Self {
        &value.inner
    }
}

impl TryFrom<Composite> for StringObject {
    type Error = crate::Error;

    fn try_from(value: Composite) -> Result<Self, Self::Error> {
        match value {
            Composite::String(s) => Ok(s),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }
}

impl<'a> TryFrom<&'a Composite> for &'a StringObject {
    type Error = crate::Error;

    fn try_from(value: &'a Composite) -> Result<Self, Self::Error> {
        match value {
            Composite::String(s) => Ok(s),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }
}

#[derive(Clone)]
pub struct NameObject {
    inner: String,
    mode: Mode,
}

impl NameObject {
    pub fn new<S>(value: S, mode: Mode) -> Self
    where
        S: Into<String>,
    {
        Self {
            inner: value.into(),
            mode,
        }
    }
}

impl<'a> From<&'a NameObject> for &'a str {
    fn from(value: &'a NameObject) -> Self {
        &value.inner
    }
}

impl PartialEq<str> for NameObject {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}
