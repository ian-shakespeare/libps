use std::{collections::HashMap, hash::Hash};

use crate::{
    array::ArrayObject, dictionary::DictionaryObject, gstate::GStateObject, object::Access,
    packedarray::PackedArrayObject, save::SaveObject, string::StringObject, Error, ErrorKind,
    FileObject, Mode,
};

pub type Address = usize;

#[derive(Clone)]
pub struct Composite {
    pub address: Address,
    pub access: Access,
    pub mode: Mode,
}

impl Hash for Composite {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}

impl PartialEq for Composite {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode && self.address == other.address
    }
}

pub enum CompositeValue {
    Array(ArrayObject),
    Dictionary(DictionaryObject),
    File(FileObject),
    GState(GStateObject),
    PackedArray(PackedArrayObject),
    Save(SaveObject),
    String(StringObject),
}

impl<'a> TryFrom<&'a CompositeValue> for &'a ArrayObject {
    type Error = Error;

    fn try_from(value: &'a CompositeValue) -> Result<Self, Self::Error> {
        match value {
            CompositeValue::Array(array) => Ok(array),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array object")),
        }
    }
}

impl<'a> TryFrom<&'a CompositeValue> for &'a DictionaryObject {
    type Error = Error;

    fn try_from(value: &'a CompositeValue) -> Result<Self, Self::Error> {
        match value {
            CompositeValue::Dictionary(dict) => Ok(dict),
            _ => {
                Err(Error::new(
                    ErrorKind::TypeCheck,
                    "expected dictionary object",
                ))
            },
        }
    }
}

impl<'a> TryFrom<&'a CompositeValue> for &'a FileObject {
    type Error = Error;

    fn try_from(value: &'a CompositeValue) -> Result<Self, Self::Error> {
        match value {
            CompositeValue::File(file) => Ok(file),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected file object")),
        }
    }
}

impl<'a> TryFrom<&'a CompositeValue> for &'a GStateObject {
    type Error = Error;

    fn try_from(value: &'a CompositeValue) -> Result<Self, Self::Error> {
        match value {
            CompositeValue::GState(gstate) => Ok(gstate),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected gstate object")),
        }
    }
}

impl<'a> TryFrom<&'a CompositeValue> for &'a PackedArrayObject {
    type Error = Error;

    fn try_from(value: &'a CompositeValue) -> Result<Self, Self::Error> {
        match value {
            CompositeValue::PackedArray(array) => Ok(array),
            _ => {
                Err(Error::new(
                    ErrorKind::TypeCheck,
                    "expected packedarray object",
                ))
            },
        }
    }
}

impl<'a> TryFrom<&'a CompositeValue> for &'a SaveObject {
    type Error = Error;

    fn try_from(value: &'a CompositeValue) -> Result<Self, Self::Error> {
        match value {
            CompositeValue::Save(save) => Ok(save),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected save object")),
        }
    }
}

impl<'a> TryFrom<&'a CompositeValue> for &'a StringObject {
    type Error = Error;

    fn try_from(value: &'a CompositeValue) -> Result<Self, Self::Error> {
        match value {
            CompositeValue::String(string) => Ok(string),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string object")),
        }
    }
}

impl From<StringObject> for CompositeValue {
    fn from(value: StringObject) -> Self {
        Self::String(value)
    }
}

pub struct Container<T> {
    inner: HashMap<Address, T>,
    next_addr: Address,
}

impl<T> Container<T> {
    pub fn insert(&mut self, value: T) -> Address {
        let addr = self.next_addr;
        let _ = self.inner.insert(addr, value);
        self.next_addr += 1;

        addr
    }

    pub fn get(&self, address: Address) -> Option<&T> {
        self.inner.get(&address)
    }

    pub fn get_mut(&mut self, address: Address) -> Option<&mut T> {
        self.inner.get_mut(&address)
    }

    pub fn remove(&mut self, address: Address) -> Option<T> {
        self.inner.remove(&address)
    }
}

impl<T> Default for Container<T> {
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
            next_addr: 0,
        }
    }
}
