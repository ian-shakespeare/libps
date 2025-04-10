use core::str;

use crate::{Error, ErrorKind};

use super::Composite;

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
