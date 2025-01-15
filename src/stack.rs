#[derive(Debug)]
pub struct Stack<T> {
    data: Vec<T>,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Stack<T>> for Vec<T> {
    fn from(value: Stack<T>) -> Self {
        value.data
    }
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn top(&self) -> Option<&T> {
        self.data.last()
    }

    pub fn search<C>(&self, condition: C) -> Option<&T>
    where
        C: Fn(&T) -> bool,
    {
        self.data.iter().rev().find(|&value| condition(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut stack: Stack<i32> = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(vec![1, 2, 3], Vec::from(stack));
    }

    #[test]
    fn test_pop() {
        let mut stack: Stack<i32> = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(Some(3), stack.pop());
        assert_eq!(vec![1, 2], Vec::from(stack));
    }

    #[test]
    fn test_top() {
        let mut stack: Stack<i32> = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(Some(3), stack.top().copied());
        assert_eq!(vec![1, 2, 3], Vec::from(stack));
    }

    #[test]
    fn test_search() {
        let mut stack: Stack<i32> = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(Some(2), stack.search(|v| *v == 2).copied());
    }
}
