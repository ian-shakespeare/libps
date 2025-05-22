use std::{
    cmp::PartialEq,
    collections::LinkedList,
    hash::{Hash, Hasher, SipHasher},
};

const DEFAULT_CAPACITY: usize = 256;

#[derive(Debug)]
pub(crate) struct Map<K, V>
where
    K: Hash + PartialEq,
    V: Clone,
{
    inner: Vec<LinkedList<(K, V)>>,
}

impl<K, V> Map<K, V>
where
    K: Hash + PartialEq,
    V: Clone,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut inner = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            inner.push(LinkedList::new());
        }

        Self { inner }
    }
}

impl<K, V> Map<K, V>
where
    K: Hash + PartialEq,
    V: Clone,
{
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut state = SipHasher::new();
        key.hash(&mut state);

        let index = (state.finish() as usize) % self.inner.len();
        for (found, value) in &self.inner[index] {
            if found == key {
                return Some(value);
            }
        }

        None
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let mut state = SipHasher::new();
        key.hash(&mut state);

        let index = (state.finish() as usize) % self.inner.len();
        for (found, value) in &mut self.inner[index] {
            if found == key {
                return Some(value);
            }
        }

        None
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        MapIterator::from(self)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut state = SipHasher::new();
        key.hash(&mut state);

        let index = (state.finish() as usize) % self.inner.len();
        match self.inner[index]
            .iter_mut()
            .find(|(found, _)| found == &key)
        {
            Some((_, found)) => {
                let old = found.clone();
                *found = value;

                Some(old)
            },
            None => {
                self.inner[index].push_back((key, value));

                None
            },
        }
    }
}

impl<K, V> Default for Map<K, V>
where
    K: Hash + PartialEq,
    V: Clone,
{
    fn default() -> Self {
        let mut inner = Vec::with_capacity(DEFAULT_CAPACITY);
        for _ in 0..DEFAULT_CAPACITY {
            inner.push(LinkedList::new())
        }

        Self { inner }
    }
}

struct MapIterator<'a, K, V>
where
    K: Hash + PartialEq,
    V: Clone,
{
    inner: &'a Map<K, V>,
    inner_cursor: usize,
    outer_cursor: usize,
}

impl<'a, K, V> From<&'a Map<K, V>> for MapIterator<'a, K, V>
where
    K: Hash + PartialEq,
    V: Clone,
{
    fn from(value: &'a Map<K, V>) -> Self {
        Self {
            inner: value,
            inner_cursor: 0,
            outer_cursor: 0,
        }
    }
}

impl<'a, K: Hash + PartialEq, V: Clone> Iterator for MapIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
