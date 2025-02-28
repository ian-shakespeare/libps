use crate::access::Access;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Executable,
    Literal,
}

#[derive(Clone, Copy, Debug)]
pub struct Composite {
    pub access: Access,
    pub key: usize,
    pub mode: Mode,
}

/*
use std::collections::HashMap;

use crate::{Error, ErrorKind, Object};

#[derive(Clone, Copy, Default)]
pub enum Access {
    #[default]
    Unlimited,
    ReadOnly,
    ExecuteOnly,
    None,
}

impl Access {
    pub fn is_read_only(&self) -> bool {
        matches!(self, Access::ReadOnly)
    }

    pub fn is_exec_only(&self) -> bool {
        matches!(self, Access::ExecuteOnly)
    }

    pub fn has_no_access(&self) -> bool {
        matches!(self, Access::None)
    }

    pub fn is_writeable(&self) -> bool {
        matches!(self, Access::Unlimited)
    }

    pub fn is_readable(&self) -> bool {
        self.is_writeable() || self.is_read_only()
    }
}

#[derive(Clone)]
enum CompositeInner {
    Array(Vec<Object>),
    String(String),
    Dictionary(HashMap<String, Object>),
}

#[derive(Clone)]
pub struct Composite {
    pub access: Access,
    inner: CompositeInner,
}

impl From<HashMap<String, Object>> for Composite {
    fn from(value: HashMap<String, Object>) -> Self {
        Self {
            access: Access::default(),
            inner: CompositeInner::Dictionary(value),
        }
    }
}

impl From<String> for Composite {
    fn from(value: String) -> Self {
        Self {
            access: Access::default(),
            inner: CompositeInner::String(value),
        }
    }
}

impl From<Vec<Object>> for Composite {
    fn from(value: Vec<Object>) -> Self {
        Self {
            access: Access::default(),
            inner: CompositeInner::Array(value),
        }
    }
}

impl Composite {
    pub fn array(&self) -> crate::Result<&Vec<Object>> {
        match self.inner {
            CompositeInner::Array(ref a) => Ok(a),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    pub fn array_mut(&mut self) -> crate::Result<&mut Vec<Object>> {
        match self.inner {
            CompositeInner::Array(ref mut a) => Ok(a),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    pub fn dict(&self) -> crate::Result<&HashMap<String, Object>> {
        match self.inner {
            CompositeInner::Dictionary(ref d) => Ok(d),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }

    pub fn dict_mut(&mut self) -> crate::Result<&mut HashMap<String, Object>> {
        match self.inner {
            CompositeInner::Dictionary(ref mut d) => Ok(d),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }

    pub fn string(&self) -> crate::Result<&str> {
        match self.inner {
            CompositeInner::String(ref s) => Ok(s),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }

    pub fn string_mut(&mut self) -> crate::Result<&mut String> {
        match self.inner {
            CompositeInner::String(ref mut s) => Ok(s),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }

    pub fn len(&self) -> usize {
        match &self.inner {
            CompositeInner::Array(array) => array.len(),
            CompositeInner::Dictionary(dict) => dict.len(),
            CompositeInner::String(string) => string.len(),
        }
    }
}
*/
