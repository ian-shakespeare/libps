use std::{fmt, hash, str};

use crate::object::Mode;

#[derive(Clone, Debug)]
pub struct NameObject {
    inner: Vec<u8>,
    mode: Mode,
}

impl NameObject {
    pub fn new(value: &str, mode: Mode) -> Self {
        Self {
            inner: value.bytes().collect(),
            mode,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn value(&self) -> &[u8] {
        &self.inner
    }
}

impl fmt::Display for NameObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { str::from_utf8_unchecked(&self.inner) };

        s.fmt(f)
    }
}

impl<'a> From<&'a str> for NameObject {
    fn from(value: &'a str) -> Self {
        Self::new(value, Mode::default())
    }
}

impl hash::Hash for NameObject {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl PartialEq for NameObject {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialEq<str> for NameObject {
    fn eq(&self, other: &str) -> bool {
        self.inner == other.as_bytes()
    }
}
