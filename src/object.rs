use core::str;
use std::collections::HashMap;

use crate::{context::Context, Error, ErrorKind};

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct ArrayObject {
    access: Access,
    pub mode: Mode,
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

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get(&self, index: usize) -> crate::Result<&Object> {
        self.inner
            .get(index)
            .ok_or(Error::from(ErrorKind::RangeCheck))
    }

    pub fn get_mut(&mut self, index: usize) -> crate::Result<&mut Object> {
        self.inner
            .get_mut(index)
            .ok_or(Error::from(ErrorKind::RangeCheck))
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

impl<'a> TryFrom<&'a mut Composite> for &'a mut ArrayObject {
    type Error = crate::Error;

    fn try_from(value: &'a mut Composite) -> Result<Self, Self::Error> {
        match value {
            Composite::Array(a) => Ok(a),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DictionaryObject {
    access: Access,
    inner: HashMap<String, Object>,
    mode: Mode,
}

impl DictionaryObject {
    pub fn new(value: HashMap<String, Object>, access: Access, mode: Mode) -> Self {
        Self {
            inner: value,
            access,
            mode,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            access: Access::default(),
            inner: HashMap::with_capacity(capacity),
            mode: Mode::default(),
        }
    }

    pub fn access(&self) -> Access {
        self.access
    }

    pub fn get(&self, key: &str) -> crate::Result<&Object> {
        self.inner
            .get(key)
            .ok_or(Error::new(ErrorKind::Undefined, key))
    }

    pub fn insert<S>(&mut self, key: S, obj: Object) -> Option<Object>
    where
        S: Into<String>,
    {
        self.inner.insert(key.into(), obj)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Object> {
        self.inner.remove(key)
    }
}

impl From<HashMap<String, Object>> for DictionaryObject {
    fn from(value: HashMap<String, Object>) -> Self {
        Self {
            access: Access::default(),
            mode: Mode::default(),
            inner: value,
        }
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

impl<'a> TryFrom<&'a mut Composite> for &'a mut DictionaryObject {
    type Error = crate::Error;

    fn try_from(value: &'a mut Composite) -> Result<Self, Self::Error> {
        match value {
            Composite::Dictionary(d) => Ok(d),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StringObject {
    inner: Vec<u8>,
}

impl StringObject {
    pub fn get(&self, index: usize) -> crate::Result<&u8> {
        self.inner
            .get(index)
            .ok_or(Error::from(ErrorKind::RangeCheck))
    }

    pub fn get_mut(&mut self, index: usize) -> crate::Result<&mut u8> {
        self.inner
            .get_mut(index)
            .ok_or(Error::from(ErrorKind::RangeCheck))
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl From<String> for StringObject {
    fn from(value: String) -> Self {
        let value = value.into_bytes();
        Self { inner: value }
    }
}

impl<'a> From<&'a StringObject> for &'a str {
    fn from(value: &'a StringObject) -> Self {
        unsafe { str::from_utf8_unchecked(&value.inner) }
    }
}

impl IntoIterator for StringObject {
    type Item = u8;
    type IntoIter = <Vec<u8> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl PartialEq for StringObject {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialEq<str> for StringObject {
    fn eq(&self, other: &str) -> bool {
        let self_str: &str = self.into();
        self_str == other
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

impl<'a> TryFrom<&'a mut Composite> for &'a mut StringObject {
    type Error = crate::Error;

    fn try_from(value: &'a mut Composite) -> Result<Self, Self::Error> {
        match value {
            Composite::String(s) => Ok(s),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }
}

#[derive(Clone, Debug)]
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

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl AsRef<str> for NameObject {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl<'a> From<&'a NameObject> for &'a str {
    fn from(value: &'a NameObject) -> Self {
        &value.inner
    }
}

impl<'a> From<&'a str> for NameObject {
    fn from(value: &'a str) -> Self {
        Self {
            inner: value.to_string(),
            mode: Mode::Executable,
        }
    }
}

impl PartialEq<str> for NameObject {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<NameObject> for NameObject {
    fn eq(&self, other: &NameObject) -> bool {
        self.inner == other.inner
    }
}
