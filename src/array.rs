use std::slice;

use crate::object::Object;

#[derive(Clone)]
pub struct ArrayObject {
    inner: Vec<Object>,
}

impl ArrayObject {
    pub fn new(objs: Vec<Object>) -> Self {
        Self { inner: objs }
    }

    pub fn iter(&self) -> slice::Iter<'_, Object> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}
