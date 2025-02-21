use std::collections::HashMap;

use crate::{composite::Composite, Error, ErrorKind, Object};

#[derive(Default)]
pub struct VirtualMemory {
    data: Vec<Composite>,
    keys: Vec<Option<usize>>,
}

impl VirtualMemory {
    pub fn get(&self, index: usize) -> crate::Result<&Composite> {
        let key = self.get_key(index)?;

        Ok(&self.data[key])
    }

    pub fn get_mut(&mut self, index: usize) -> crate::Result<&mut Composite> {
        let key = self.get_key(index)?;

        Ok(&mut self.data[key])
    }

    pub fn get_array(&self, index: usize) -> crate::Result<&Vec<Object>> {
        let composite = self.get(index)?;

        composite.array()
    }

    pub fn get_array_mut(&mut self, index: usize) -> crate::Result<&mut Vec<Object>> {
        let composite = self.get_mut(index)?;

        composite.array_mut()
    }

    pub fn get_dict(&self, index: usize) -> crate::Result<&HashMap<String, Object>> {
        let composite = self.get(index)?;

        composite.dict()
    }

    pub fn get_dict_mut(&mut self, index: usize) -> crate::Result<&mut HashMap<String, Object>> {
        let composite = self.get_mut(index)?;

        composite.dict_mut()
    }

    pub fn get_string(&self, index: usize) -> crate::Result<&str> {
        let composite = self.get(index)?;

        composite.string()
    }

    pub fn get_string_mut(&mut self, index: usize) -> crate::Result<&mut String> {
        let composite = self.get_mut(index)?;

        composite.string_mut()
    }

    pub fn insert<C: Into<Composite>>(&mut self, value: C) -> usize {
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
        let mut vm = VirtualMemory::default();

        vm.insert("string1".to_string());
        vm.insert(vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
        ]);

        let mut dict = HashMap::new();
        dict.insert("key".to_string(), Object::Boolean(true));
        vm.insert(dict);

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
        let mut vm = VirtualMemory::default();

        let idx = vm.insert("string1".to_string());
        assert_eq!("string1", vm.get(idx)?.string()?);

        let input = vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)];
        let idx = vm.insert(input.clone());
        assert_eq!(input, vm.get(idx)?.array()?.clone());

        let mut input = HashMap::new();
        input.insert("key".to_string(), Object::Boolean(true));
        let idx = vm.insert(input.clone());
        assert_eq!(input, vm.get(idx)?.dict()?.clone());

        Ok(())
    }

    #[test]
    fn test_delete() -> Result<(), Box<dyn error::Error>> {
        let mut vm = VirtualMemory::default();

        let str_idx = vm.insert("string1".to_string());
        vm.insert(vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
        ]);

        let mut dict = HashMap::new();
        dict.insert("key".to_string(), Object::Boolean(true));
        let dict_idx = vm.insert(dict);

        vm.delete(str_idx)?;

        let key_count = vm
            .keys
            .iter()
            .fold(0, |acc, key| if key.is_some() { acc + 1 } else { acc });
        assert_eq!(2, key_count);
        assert_eq!(3, vm.keys.len());
        assert_eq!(2, vm.data.len());
        assert_eq!(Some(Some(0)), vm.keys.get(dict_idx).copied());

        vm.insert("string2".to_string());

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
        let mut vm = VirtualMemory::default();

        let idx = vm.insert("string".to_string());
        vm.delete(idx)?;

        assert_eq!(0, vm.keys.len());
        assert_eq!(0, vm.data.len());

        Ok(())
    }

    #[test]
    fn test_delete_last() -> Result<(), Box<dyn error::Error>> {
        let mut vm = VirtualMemory::default();

        vm.insert("string1".to_string());
        vm.insert(vec![
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
        ]);

        let mut dict = HashMap::new();
        dict.insert("key".to_string(), Object::Boolean(true));
        let last_idx = vm.insert(dict);

        assert_eq!(3, vm.data.len());
        assert_eq!(3, vm.keys.len());

        vm.delete(last_idx)?;
        assert_eq!(2, vm.data.len());
        assert_eq!(2, vm.keys.len());

        Ok(())
    }
}
