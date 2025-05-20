use std::{fmt, hash, str};

use crate::object::{Access, Mode};

#[derive(Debug)]
pub struct StringObject {
    access: Access,
    inner: Vec<u8>,
    pub(crate) mode: Mode,
}

impl StringObject {
    pub fn new<S: Into<String>>(value: S, access: Access, mode: Mode) -> Self {
        Self {
            inner: value.into().bytes().collect(),
            access,
            mode,
        }
    }

    pub fn access(&self) -> Access {
        self.access
    }

    pub fn value(&self) -> &[u8] {
        &self.inner
    }
}

impl fmt::Display for StringObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match str::from_utf8(&self.inner) {
            Ok(s) => s.fmt(f),
            Err(_) => "(� )".fmt(f),
        }
    }
}

impl From<&str> for StringObject {
    fn from(value: &str) -> Self {
        Self {
            access: Access::default(),
            inner: value.bytes().collect(),
            mode: Mode::default(),
        }
    }
}

impl From<String> for StringObject {
    fn from(value: String) -> Self {
        Self {
            access: Access::default(),
            inner: value.bytes().collect(),
            mode: Mode::default(),
        }
    }
}

impl From<StringObject> for Vec<u8> {
    fn from(value: StringObject) -> Self {
        value.inner
    }
}

impl hash::Hash for StringObject {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl PartialEq for StringObject {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
