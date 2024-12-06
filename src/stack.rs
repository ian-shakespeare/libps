pub struct Stack<T> {
    data: Vec<T>,
}

impl<'a, T> Stack<T> {
    pub fn new() -> Stack<T> {
        return Stack { data: Vec::new() };
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn pop_two(&mut self) -> Option<(T, T)> {
        match self.pop() {
            Some(rhs) => match self.pop() {
                Some(lhs) => return Some((lhs, rhs)),
                None => None,
            },
            None => None,
        }
    }

    pub fn top(&'a self) -> Option<&'a T> {
        self.data.last()
    }
}
