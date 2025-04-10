use crate::{Error, ErrorKind};

use super::{Access, Composite, Mode, Object};

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
