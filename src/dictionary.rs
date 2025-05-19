use std::{collections::HashMap, fmt};

use crate::{Mode, Object};

#[derive(Debug, Default)]
pub struct DictionaryObject {
    inner: HashMap<Object, Object>,
    mode: Mode,
}

impl DictionaryObject {
    pub fn insert(&mut self, k: Object, v: Object) -> Option<Object> {
        self.inner.insert(k, v)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, Object, Object> {
        self.inner.iter()
    }

    pub fn get(&self, k: &Object) -> Option<&Object> {
        self.inner.get(k)
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
}

impl fmt::Display for DictionaryObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "<< ".fmt(f)?;
        for (k, v) in self.iter() {
            k.fmt(f)?;
            " ".fmt(f)?;
            v.fmt(f)?;
            " ".fmt(f)?;
        }
        ">>".fmt(f)?;

        Ok(())
    }
}
