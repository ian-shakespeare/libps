use std::{
    alloc::{Allocator, Global},
    cmp::PartialEq,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Index,
};

const DEFAULT_CAPACITY: usize = 16;
const MAX_ATTEMPTS: usize = 1024;

#[derive(Debug)]
pub(crate) struct HashMap<K, V, A = Global>
where
    K: Hash + PartialEq,
    V: Clone,
    A: Allocator,
{
    inner: Vec<Option<(K, V)>, A>,
    len: usize,
}

impl<K, V> HashMap<K, V, Global>
where
    K: Hash + PartialEq,
    V: Clone,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = (capacity * 13) / 10;
        let mut inner = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            inner.push(None);
        }

        Self { len: 0, inner }
    }
}

impl<K, V, A> HashMap<K, V, A>
where
    K: Hash + PartialEq,
    V: Clone,
    A: Allocator,
{
    pub fn new_in(alloc: A) -> Self {
        Self::with_capacity_in(DEFAULT_CAPACITY, alloc)
    }

    pub fn with_capacity_in(capacity: usize, alloc: A) -> Self {
        let capacity = (capacity * 13) / 10;
        let mut inner = Vec::with_capacity_in(capacity, alloc);
        for _ in 0..capacity {
            inner.push(None);
        }

        Self { len: 0, inner }
    }
}

impl<K, V, A> HashMap<K, V, A>
where
    K: Hash + PartialEq,
    V: Clone,
    A: Allocator,
{
    pub fn capacity(&self) -> usize {
        self.inner.len()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let initial_index = self.hash_index(&key);

        let mut attempt = 1;
        let mut index = initial_index;
        while self.inner[index]
            .as_ref()
            .is_some_and(|(found, _)| found != key)
        {
            if attempt > MAX_ATTEMPTS {
                panic!("exceeded the maximum number of probe attempts");
            }

            index = (initial_index + (attempt * attempt)) % self.capacity();
            attempt += 1;
        }

        match &self.inner[index] {
            Some((_, value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let initial_index = self.hash_index(&key);

        let mut attempt = 1;
        let mut index = initial_index;
        while self.inner[index]
            .as_ref()
            .is_some_and(|(found, _)| found != key)
        {
            if attempt > MAX_ATTEMPTS {
                panic!("exceeded the maximum number of probe attempts");
            }

            index = (initial_index + (attempt * attempt)) % self.capacity();
            attempt += 1;
        }

        match &mut self.inner[index] {
            Some((_, value)) => Some(value),
            _ => None,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let initial_index = self.hash_index(&key);

        let mut attempt = 1;
        let mut index = initial_index;
        while self.inner[index]
            .as_ref()
            .is_some_and(|(found, _)| found != &key)
        {
            if attempt > MAX_ATTEMPTS {
                panic!("exceeded the maximum number of probe attempts");
            }

            index = (initial_index + (attempt * attempt)) % self.capacity();
            attempt += 1;
        }

        match &mut self.inner[index] {
            Some((_, slot)) => {
                let found = slot.clone();
                *slot = value;

                Some(found)
            },
            None => {
                self.inner[index] = Some((key, value));
                self.len += 1;

                // reserve len * 1.3 additional space
                if let Some(additional) = ((self.len() * 13) / 10).checked_sub(self.capacity()) {
                    self.reserve(additional);
                }

                None
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        HashMapIterator::from(self)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let initial_index = self.hash_index(key);

        let mut attempt = 1;
        let mut index = initial_index;
        while self.inner[index]
            .as_ref()
            .is_some_and(|(found, _)| found != key)
        {
            if attempt > MAX_ATTEMPTS {
                panic!("exceeded the maximum number of probe attempts");
            }

            index = (initial_index + (attempt * attempt)) % self.capacity();
            attempt += 1;
        }

        let value = match &self.inner[index] {
            Some((_, value)) => {
                self.len -= 1;

                Some(value.clone())
            },
            None => None,
        };
        self.inner[index] = None;

        value
    }

    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
        for _ in 0..additional {
            self.inner.push(None);
        }
    }

    fn hash_index(&self, key: &K) -> usize {
        let mut state = DefaultHasher::new();
        key.hash(&mut state);

        (state.finish() as usize) % self.capacity()
    }
}

impl<K, V> Default for HashMap<K, V, Global>
where
    K: Hash + PartialEq,
    V: Clone,
{
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }
}

impl<K, V, I> From<I> for HashMap<K, V, Global>
where
    K: Hash + PartialEq,
    V: Clone,
    I: IntoIterator<Item = (K, V)>,
{
    fn from(value: I) -> Self {
        Self::from_iter(value)
    }
}

impl<K, V> FromIterator<(K, V)> for HashMap<K, V, Global>
where
    K: Hash + PartialEq,
    V: Clone,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        iter.into_iter().fold(Self::new(), |mut map, (k, v)| {
            map.insert(k, v);

            map
        })
    }
}

impl<'a, K, V, A> Index<usize> for HashMap<K, V, A>
where
    K: Hash + PartialEq,
    V: Clone,
    A: Allocator,
{
    type Output = Option<(K, V)>;

    fn index(&self, index: usize) -> &Self::Output {
        let index = index % self.capacity();
        &self.inner[index]
    }
}

struct HashMapIterator<'a, K, V, A>
where
    K: Hash + PartialEq,
    V: Clone,
    A: Allocator,
{
    cursor: usize,
    inner: &'a HashMap<K, V, A>,
}

impl<'a, K, V, A> From<&'a HashMap<K, V, A>> for HashMapIterator<'a, K, V, A>
where
    K: Hash + PartialEq,
    V: Clone,
    A: Allocator,
{
    fn from(value: &'a HashMap<K, V, A>) -> Self {
        Self {
            cursor: 0,
            inner: value,
        }
    }
}

impl<'a, K, V, A> Iterator for HashMapIterator<'a, K, V, A>
where
    K: Hash + PartialEq,
    V: Clone,
    A: Allocator,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            return None;
        }

        while self.inner[self.cursor].is_none() {
            if self.cursor >= self.inner.capacity() {
                return None;
            }

            self.cursor += 1;
        }

        match self.inner[self.cursor].as_ref() {
            Some((k, v)) => Some((k, v)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut map: HashMap<&str, i32> = HashMap::new();
        assert!(map.capacity() >= DEFAULT_CAPACITY);
        assert_eq!(0, map.len());

        let existing = map.insert("key", 1);
        assert!(existing.is_none());
        assert_eq!(1, map.len());
    }

    #[test]
    fn test_insert_increase_capacity() {
        // should stay at default capacity
        let mut map = HashMap::from([
            ("key1", 0),
            ("key2", 0),
            ("key3", 0),
            ("key4", 0),
            ("key5", 0),
            ("key6", 0),
            ("key7", 0),
            ("key8", 0),
            ("key9", 0),
            ("key10", 0),
            ("key11", 0),
            ("key12", 0),
            ("key13", 0),
            ("key14", 0),
            ("key15", 0),
            ("key16", 0),
        ]);

        let starting_capacity = map.capacity();
        map.insert("key17", 0);
        let ending_capacity = map.capacity();

        assert_ne!(starting_capacity, ending_capacity)
    }

    #[test]
    fn test_insert_shadow() {
        let mut map = HashMap::from([("key", 0)]);
        assert_eq!(1, map.len());

        let existing = map.insert("key", 1);
        assert_eq!(Some(0), existing);
        assert_eq!(1, map.len());
    }

    #[test]
    fn test_get() {
        let map = HashMap::from([("key1", 0), ("key2", 1), ("key3", 2)]);
        let starting_len = map.len();

        assert_eq!(Some(&0), map.get(&"key1"));
        assert_eq!(Some(&1), map.get(&"key2"));
        assert_eq!(Some(&2), map.get(&"key3"));
        assert_eq!(starting_len, map.len());
    }

    #[test]
    fn test_get_nonexistant() {
        let map = HashMap::from([("key1", 0), ("key2", 1), ("key3", 2)]);
        let starting_len = map.len();

        assert_eq!(None, map.get(&"bad"));
        assert_eq!(starting_len, map.len());
    }

    #[test]
    fn test_get_mut() {
        let mut map = HashMap::from([("key1", 0), ("key2", 1), ("key3", 2)]);
        let starting_len = map.len();

        {
            let value = map.get_mut(&"key1");
            assert!(value.is_some());

            let slot = value.unwrap();
            *slot = 10;
            assert_eq!(starting_len, map.len());
        }

        assert_eq!(Some(&10), map.get(&"key1"));
    }

    #[test]
    fn test_get_mut_nonexistant() {
        let map = HashMap::from([("key1", 0), ("key2", 1), ("key3", 2)]);
        let starting_len = map.len();

        assert_eq!(None, map.get(&"bad"));
        assert_eq!(starting_len, map.len());
    }

    #[test]
    fn test_remove() {
        let mut map = HashMap::from([("key1", 0), ("key2", 1), ("key3", 2)]);
        let starting_len = map.len();

        assert_eq!(Some(0), map.remove(&"key1"));
        assert_eq!(starting_len - 1, map.len());
    }

    #[test]
    fn test_remove_nonexistant() {
        let mut map = HashMap::from([("key1", 0), ("key2", 1), ("key3", 2)]);
        let starting_len = map.len();

        assert_eq!(None, map.remove(&"bad"));
        assert_eq!(starting_len, map.len());
    }
}
