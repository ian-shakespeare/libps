use std::collections::HashMap;

use crate::Object;

#[derive(Clone, Default)]
pub struct DictionaryObject {
    inner: HashMap<Object, Object>,
}

impl DictionaryObject {
    pub fn insert(&mut self, k: Object, v: Object) -> Option<Object> {
        self.inner.insert(k, v)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Object, &Object)> {
        self.inner.iter()
    }

    pub fn get(&self, k: &Object) -> Option<&Object> {
        self.inner.get(k)
    }
}
