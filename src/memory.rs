use crate::{Error, ErrorKind};

pub struct VirtualMemory<T> {
    data: Vec<T>,
    keys: Vec<Option<usize>>,
}

impl<T> VirtualMemory<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            keys: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> crate::Result<&T> {
        let key = self.get_key(index)?;

        Ok(&self.data[key])
    }

    pub fn get_mut(&mut self, index: usize) -> crate::Result<&mut T> {
        let key = self.get_key(index)?;

        Ok(&mut self.data[key])
    }

    pub fn insert<I: Into<T>>(&mut self, value: I) -> usize {
        let key = self.data.len();
        self.data.push(value.into());

        let index = match self.keys.iter().enumerate().find(|(_, k)| k.is_none()) {
            Some((index, ..)) => index,
            None => {
                let index = self.keys.len();
                self.keys.push(None);

                index
            },
        };

        self.keys[index] = Some(key);

        index
    }

    pub fn delete(&mut self, index: usize) -> crate::Result<()> {
        if self.data.len() == 1 {
            self.keys.clear();
            self.data.clear();
            return Ok(());
        }

        let deleted_key = self.get_key(index)?;

        if deleted_key == self.data.len() - 1 {
            self.data.pop();
        } else {
            let end = self.data.len() - 1;

            self.data[deleted_key] = self.data.pop().ok_or(Error::from(ErrorKind::VmError))?;

            let (moved_key_index, ..) = self
                .keys
                .iter()
                .enumerate()
                .find(|(_, key)| key.is_some_and(|key| key == end))
                .ok_or(Error::from(ErrorKind::VmError))?;

            self.keys[moved_key_index] = Some(deleted_key);
        }

        if index == self.keys.len() - 1 {
            self.keys.pop();
        } else {
            self.keys[index] = None;
        }

        Ok(())
    }

    fn get_key(&self, index: usize) -> crate::Result<usize> {
        match self.keys.get(index) {
            Some(Some(key)) => Ok(*key),
            _ => Err(Error::from(ErrorKind::VmError)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    #[test]
    fn test_insert() -> Result<(), Box<dyn error::Error>> {
        let mut vm: VirtualMemory<&str> = VirtualMemory::new();

        vm.insert("string1");
        vm.insert("string2");
        vm.insert("string3");

        assert_eq!(3, vm.data.len());
        assert_eq!(3, vm.keys.len());

        for key in vm.keys {
            let key = key.ok_or("expected key")?;

            assert!(vm.data.get(key).is_some());
        }

        Ok(())
    }

    #[test]
    fn test_get() -> Result<(), Box<dyn error::Error>> {
        let mut vm: VirtualMemory<&str> = VirtualMemory::new();

        let idx = vm.insert("string1");
        assert_eq!("string1", *vm.get(idx)?);

        let idx = vm.insert("string2");
        assert_eq!("string2", *vm.get(idx)?);

        let idx = vm.insert("string3");
        assert_eq!("string3", *vm.get(idx)?);

        Ok(())
    }

    #[test]
    fn test_delete() -> Result<(), Box<dyn error::Error>> {
        let mut vm: VirtualMemory<&str> = VirtualMemory::new();

        let idx1 = vm.insert("string1");
        vm.insert("string2");
        let idx2 = vm.insert("string3");

        vm.delete(idx1)?;

        let key_count = vm
            .keys
            .iter()
            .fold(0, |acc, key| if key.is_some() { acc + 1 } else { acc });
        assert_eq!(2, key_count);
        assert_eq!(3, vm.keys.len());
        assert_eq!(2, vm.data.len());
        assert_eq!(Some(Some(0)), vm.keys.get(idx2).copied());

        vm.insert("string4");

        let key_count = vm
            .keys
            .iter()
            .fold(0, |acc, key| if key.is_some() { acc + 1 } else { acc });
        assert_eq!(3, key_count);
        assert_eq!(3, vm.keys.len());
        assert_eq!(3, vm.data.len());

        Ok(())
    }

    #[test]
    fn test_delete_single() -> Result<(), Box<dyn error::Error>> {
        let mut vm: VirtualMemory<&str> = VirtualMemory::new();

        let idx = vm.insert("string");
        vm.delete(idx)?;

        assert_eq!(0, vm.keys.len());
        assert_eq!(0, vm.data.len());

        Ok(())
    }

    #[test]
    fn test_delete_last() -> Result<(), Box<dyn error::Error>> {
        let mut vm: VirtualMemory<&str> = VirtualMemory::new();

        vm.insert("string1");
        vm.insert("string2");

        let last_idx = vm.insert("string3");

        assert_eq!(3, vm.data.len());
        assert_eq!(3, vm.keys.len());

        vm.delete(last_idx)?;
        assert_eq!(2, vm.data.len());
        assert_eq!(2, vm.keys.len());

        Ok(())
    }
}
