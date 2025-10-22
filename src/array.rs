use std::slice;

use crate::object::{Access, Mode, Object};

// max capacity: 65535 (u16)
pub struct ArrayObject {
    access: Access,
    inner: Vec<Object>,
    pub(crate) mode: Mode,
}

impl ArrayObject {
    pub fn new(objs: Vec<Object>, access: Access, mode: Mode) -> Self {
        Self {
            inner: objs,
            access,
            mode,
        }
    }

    pub fn access(&self) -> Access {
        self.access
    }

    pub fn iter(&self) -> slice::Iter<'_, Object> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl From<ArrayObject> for Vec<u8> {
    fn from(value: ArrayObject) -> Self {
        todo!()
    }
}
