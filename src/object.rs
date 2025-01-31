use std::collections;

use crate::{Error, ErrorKind};

#[derive(Default)]
pub enum Access {
    #[default]
    Unlimited,
    ReadOnly,
    ExecuteOnly,
    None,
}

pub struct Composite<T> {
    pub inner: T,
    pub access: Access,
    pub len: usize,
}

impl<T> Composite<T> {
    pub fn is_read_only(&self) -> bool {
        matches!(self.access, Access::ReadOnly)
    }

    pub fn is_exec_only(&self) -> bool {
        matches!(self.access, Access::ExecuteOnly)
    }
}

#[derive(Clone, Debug)]
pub enum Object {
    Integer(i32),
    Real(f64),
    Boolean(bool),
    Array(usize),
    PackedArray(usize),
    String(usize),
    Dictionary(usize),
    Literal(String),
    Name(String),
    File,
    Mark,
    Null,
    Save,
    FontId, // TODO: Figure out what these things are
    GState,
}

impl From<Object> for String {
    fn from(value: Object) -> Self {
        match value {
            Object::Integer(value) => value.to_string(),
            Object::Real(value) => value.to_string(),
            Object::Boolean(value) => value.to_string(),
            Object::Name(value) => value,
            _ => "".to_string(),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Integer(value) => match other {
                Self::Integer(other_value) => value == other_value,
                Self::Real(other_value) => f64::from(*value) == *other_value,
                _ => false,
            },
            Self::Real(value) => match other {
                Self::Integer(other_value) => *value == f64::from(*other_value),
                Self::Real(other_value) => value == other_value,
                _ => false,
            },
            Self::Boolean(value) => match other {
                Self::Boolean(other_value) => value == other_value,
                _ => false,
            },
            Self::Array(value) => match other {
                Self::Array(other_value) => value == other_value,
                _ => false,
            },
            Self::PackedArray(value) => match other {
                Self::PackedArray(other_value) => value == other_value,
                _ => false,
            },
            Self::String(value) => match other {
                Self::String(other_value) => value == other_value,
                _ => false,
            },
            Self::Name(value) => match other {
                Self::Name(other_value) => value == other_value,
                _ => false,
            },
            Self::Null => matches!(other, Self::Null),
            _ => false,
        }
    }
}

impl Eq for Object {}

impl Object {
    pub fn is_int(&self) -> bool {
        match self {
            Self::Integer(_) => true,
            _ => false,
        }
    }

    pub fn is_real(&self) -> bool {
        match self {
            Self::Real(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Self::String(_) => true,
            _ => false,
        }
    }

    pub fn into_int(&self) -> crate::Result<i32> {
        match self {
            Self::Integer(i) => Ok(*i),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        }
    }

    pub fn into_real(&self) -> crate::Result<f64> {
        match self {
            Self::Integer(i) => Ok(f64::from(*i)),
            Self::Real(r) => Ok(*r),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected real")),
        }
    }
}

pub struct Container<V> {
    inner: collections::HashMap<usize, V>,
    counter: usize,
}

impl<V> Container<V> {
    pub fn new() -> Self {
        Self {
            inner: collections::HashMap::new(),
            counter: 0,
        }
    }

    pub fn insert(&mut self, v: V) -> usize {
        self.counter += 1;

        let _ = self.inner.insert(self.counter, v);

        self.counter
    }

    pub fn get(&mut self, k: usize) -> crate::Result<&V> {
        match self.inner.get(&k) {
            Some(v) => Ok(v),
            None => Err(Error::from(ErrorKind::Undefined)), // TODO: VmError
        }
    }

    pub fn get_mut(&mut self, k: usize) -> crate::Result<&mut V> {
        match self.inner.get_mut(&k) {
            Some(v) => Ok(v),
            None => Err(Error::from(ErrorKind::Undefined)), // TODO: VmError
        }
    }
}
