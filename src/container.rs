use std::collections::HashMap;

pub struct Container<T> {
    count: usize,
    inner: HashMap<usize, T>,
}

impl<T> Container<T> {
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(&index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(&index)
    }

    pub fn insert<I: Into<T>>(&mut self, value: I) -> usize {
        let idx = self.count;
        self.inner.insert(idx, value.into());
        self.count += 1;

        idx
    }
}

impl<T> Default for Container<T> {
    fn default() -> Self {
        Self {
            count: 0,
            inner: HashMap::new(),
        }
    }
}
