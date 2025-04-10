use std::collections::HashMap;

use crate::{Error, ErrorKind};

use super::{Access, Composite, Mode, Object};

#[derive(Clone, Debug, Default)]
pub struct DictionaryObject {
    access: Access,
    inner: HashMap<String, Object>,
    pub mode: Mode,
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

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
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
