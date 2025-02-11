use std::collections::HashMap;

use crate::{Error, ErrorKind, Interpreter};

#[derive(Clone, Debug)]
pub enum Object {
    Integer(i32),
    Real(f64),
    Boolean(bool),
    Array(usize),
    PackedArray(usize),
    Procedure(usize),
    String(usize),
    Dictionary(usize),
    Name(String),
    Operator(fn(&mut Interpreter) -> crate::Result<()>),
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
            Object::Array(_) => "array".to_string(),
            Object::PackedArray(_) => "packedarray".to_string(),
            Object::Mark => "mark".to_string(),
            Object::Null => "null".to_string(),
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
    pub fn to_string(&self, interpreter: &Interpreter) -> crate::Result<String> {
        match self {
            Object::Integer(i) => Ok(i.to_string()),
            Object::Real(r) => Ok(r.to_string()),
            Object::String(idx) => Ok(interpreter.strings.get(*idx)?.inner.clone()),
            Object::Name(name) => Ok(name.to_string()),
            _ => Err(Error::new(
                ErrorKind::Unregistered,
                "cannot stringify object",
            )),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Integer(..) => "integer",
            Self::Real(..) => "real",
            Self::Boolean(..) => "boolean",
            Self::Name(..) => "name",
            Self::PackedArray(..) => "packedarray",
            Self::Array(..) => "array",
            Self::Procedure(..) => "procedure",
            Self::String(..) => "string",
            Self::Dictionary(..) => "dictionary",
            Self::Operator(..) => "operator",
            Self::File => "file",
            Self::Mark => "mark",
            Self::Null => "null",
            Self::Save => "save",
            Self::FontId => "fontid",
            Self::GState => "gstate",
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Integer(..))
    }

    pub fn is_real(&self) -> bool {
        matches!(self, Self::Real(..))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }

    pub fn is_mark(&self) -> bool {
        matches!(self, Self::Mark)
    }

    pub fn is_procedure(&self) -> bool {
        matches!(self, Self::Procedure(..))
    }

    pub fn into_int(&self) -> crate::Result<i32> {
        match self {
            Self::Integer(i) => Ok(*i),
            _ => Err(Error::new(
                ErrorKind::TypeCheck,
                format!("expected integer, received {}", self.name()),
            )),
        }
    }

    pub fn into_real(&self) -> crate::Result<f64> {
        match self {
            Self::Integer(i) => Ok(f64::from(*i)),
            Self::Real(r) => Ok(*r),
            _ => Err(Error::new(
                ErrorKind::TypeCheck,
                format!("expected real, received {}", self.name()),
            )),
        }
    }
}

pub struct Container<V> {
    inner: HashMap<usize, V>,
    counter: usize,
}

impl<V> Default for Container<V> {
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
            counter: 0,
        }
    }
}

impl<V> Container<V> {
    pub fn insert(&mut self, v: V) -> usize {
        self.counter += 1;

        let _ = self.inner.insert(self.counter, v);

        self.counter
    }

    pub fn get(&self, k: usize) -> crate::Result<&V> {
        match self.inner.get(&k) {
            Some(v) => Ok(v),
            None => Err(Error::from(ErrorKind::VmError)),
        }
    }

    pub fn get_mut(&mut self, k: usize) -> crate::Result<&mut V> {
        match self.inner.get_mut(&k) {
            Some(v) => Ok(v),
            None => Err(Error::from(ErrorKind::VmError)),
        }
    }
}

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

pub struct PostScriptArray {
    access: Access,
    inner: Vec<Object>,
    len: usize,
}

impl From<Vec<Object>> for PostScriptArray {
    fn from(value: Vec<Object>) -> Self {
        Self::new(value, Access::default())
    }
}

impl PostScriptArray {
    pub fn new(value: Vec<Object>, access: Access) -> Self {
        let len = value.len();

        Self {
            inner: value,
            access,
            len,
        }
    }

    pub fn value(&self) -> &Vec<Object> {
        &self.inner
    }

    pub fn value_mut(&mut self) -> &mut Vec<Object> {
        &mut self.inner
    }

    pub fn access(&self) -> Access {
        self.access
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub struct PostScriptDictionary {
    access: Access,
    capacity: usize,
    inner: HashMap<String, Object>,
    len: usize,
}

impl From<HashMap<String, Object>> for PostScriptDictionary {
    fn from(value: HashMap<String, Object>) -> Self {
        Self {
            access: Access::default(),
            capacity: value.len(),
            len: value.len(),
            inner: value,
        }
    }
}

impl PostScriptDictionary {
    pub fn new(capacity: usize, access: Access) -> Self {
        Self {
            inner: HashMap::new(),
            len: 0,
            access,
            capacity,
        }
    }

    pub fn insert(&mut self, key: String, obj: Object) -> crate::Result<()> {
        if self.len + 1 > self.capacity {
            return Err(Error::from(ErrorKind::DictFull));
        }

        self.len += 1;
        let _ = self.inner.insert(key, obj);

        Ok(())
    }

    pub fn get(&self, key: String) -> crate::Result<&Object> {
        self.inner
            .get(&key)
            .ok_or(Error::new(ErrorKind::Undefined, key))
    }

    pub fn get_mut(&mut self, key: String) -> crate::Result<&mut Object> {
        self.inner
            .get_mut(&key)
            .ok_or(Error::new(ErrorKind::Undefined, key))
    }

    pub fn access(&self) -> Access {
        self.access
    }

    pub fn set_access(&mut self, access: Access) {
        self.access = access;
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn value(&self) -> &HashMap<String, Object> {
        &self.inner
    }
}

pub struct PostScriptString {
    access: Access,
    inner: String,
    len: usize,
}

impl From<String> for PostScriptString {
    fn from(value: String) -> Self {
        Self::new(value, Access::default())
    }
}

impl PostScriptString {
    pub fn new(value: String, access: Access) -> Self {
        let len = value.len();

        Self {
            inner: value,
            access,
            len,
        }
    }

    pub fn value(&self) -> &str {
        &self.inner
    }

    pub fn access(&self) -> Access {
        self.access
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
