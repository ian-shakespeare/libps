use std::{fmt, hash, str};

#[derive(Clone, Debug)]
pub struct StringObject {
    inner: Vec<u8>,
}

impl StringObject {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self {
            inner: value.into().bytes().collect(),
        }
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
            inner: value.bytes().collect(),
        }
    }
}

impl From<String> for StringObject {
    fn from(value: String) -> Self {
        Self {
            inner: value.bytes().collect(),
        }
    }
}

impl From<Vec<u8>> for StringObject {
    fn from(value: Vec<u8>) -> Self {
        Self { inner: value }
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
