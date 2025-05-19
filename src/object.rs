use std::{cell::RefCell, fmt, hash, rc::Rc};

use crate::{
    array::ArrayObject, dictionary::DictionaryObject, file::FileObject, name::NameObject,
    operator::OperatorObject, string::StringObject, Error, ErrorKind,
};

#[derive(Clone)]
pub enum Object {
    // Simple
    Boolean(bool),
    FontID,
    Integer(i32),
    Mark,
    Name(NameObject),
    Null(Mode),
    Operator((OperatorObject, Mode)),
    Real(f32),

    // Composite
    Array(Rc<RefCell<ArrayObject>>),
    Dictionary(Rc<RefCell<DictionaryObject>>),
    File(Rc<RefCell<FileObject>>),
    GState(Rc<RefCell<GState>>),
    PackedArray(Rc<RefCell<PackedArray>>),
    Save(Rc<RefCell<Save>>),
    String(Rc<RefCell<StringObject>>),
}

impl Object {
    pub fn into_int(self) -> crate::Result<i32> {
        match self {
            Object::Integer(i) => Ok(i),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected int")),
        }
    }

    pub fn into_name(self) -> crate::Result<NameObject> {
        match self {
            Object::Name(n) => Ok(n),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected name")),
        }
    }

    pub fn into_real(self) -> crate::Result<f32> {
        match self {
            Object::Real(r) => Ok(r),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected real")),
        }
    }

    pub fn mode(&self) -> Mode {
        match self {
            Object::Name(name) => name.mode(),
            Object::Null(mode) => *mode,
            Object::Operator((_, mode)) => *mode,
            Object::Array(array) => array.borrow().mode(),
            Object::File(file) => file.borrow().mode(),
            Object::String(string) => string.borrow().mode(),
            _ => Mode::Literal,
        }
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Boolean(b) => b.fmt(f),
            Object::Integer(i) => i.fmt(f),
            Object::Mark => "mark".fmt(f),
            Object::Name(n) => n.fmt(f),
            Object::Null(_) => "null".fmt(f),
            Object::Operator((o, _)) => o.to_string().fmt(f),
            Object::Real(r) => r.fmt(f),
            Object::Array(a) => a.fmt(f),
            Object::Dictionary(d) => d.fmt(f),
            Object::String(s) => s.fmt(f),
            _ => "TODO".fmt(f),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Boolean(b) => b.fmt(f),
            Object::Integer(i) => i.fmt(f),
            Object::Mark => "mark".fmt(f),
            Object::Name(n) => n.fmt(f),
            Object::Null(_) => "null".fmt(f),
            Object::Operator((o, _)) => o.to_string().fmt(f),
            Object::Real(r) => r.fmt(f),
            Object::Array(a) => a.borrow().fmt(f),
            Object::Dictionary(d) => d.borrow().fmt(f),
            Object::String(s) => s.borrow().fmt(f),
            _ => "TODO".fmt(f),
        }
    }
}

impl Eq for Object {}

impl hash::Hash for Object {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::Boolean(b) => b.hash(state),
            Object::Integer(i) => i.hash(state),
            Object::Name(n) => n.hash(state),
            Object::Real(r) => {
                let u = r.to_bits();
                u.hash(state);
            },
            Object::Array(a) => {
                for obj in a.borrow().iter() {
                    obj.hash(state);
                }
            },
            Object::Dictionary(d) => {
                for (key, value) in d.borrow().iter() {
                    key.hash(state);
                    value.hash(state);
                }
            },
            Object::String(s) => s.borrow().hash(state),
            _ => {},
        };
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Boolean(lhs), Object::Boolean(rhs)) => lhs == rhs,
            (Object::Integer(lhs), Object::Integer(rhs)) => lhs == rhs,
            (Object::Integer(lhs), Object::Real(rhs)) => *lhs as f32 == *rhs,
            (Object::Name(lhs), Object::Name(rhs)) => lhs == rhs,
            (Object::Name(lhs), Object::String(rhs)) => lhs.value() == rhs.borrow().value(),
            (Object::Operator(lhs), Object::Operator(rhs)) => lhs == rhs,
            (Object::Real(lhs), Object::Real(rhs)) => lhs == rhs,
            (Object::Real(lhs), Object::Integer(rhs)) => *lhs == *rhs as f32,
            (Object::Array(lhs), Object::Array(rhs)) => Rc::ptr_eq(lhs, rhs),
            (Object::Dictionary(lhs), Object::Dictionary(rhs)) => Rc::ptr_eq(lhs, rhs),
            (Object::File(lhs), Object::File(rhs)) => Rc::ptr_eq(lhs, rhs),
            (Object::String(lhs), Object::String(rhs)) => lhs == rhs,
            (Object::String(lhs), Object::Name(rhs)) => lhs.borrow().value() == rhs.value(),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Access {
    #[default]
    Unlimited,
    ReadOnly,
    ExecuteOnly,
    None,
}

impl Access {
    pub fn is_writeable(&self) -> bool {
        *self == Self::Unlimited
    }

    pub fn is_readable(&self) -> bool {
        self.is_writeable() || *self == Self::ReadOnly
    }

    pub fn is_executable(&self) -> bool {
        self.is_readable() || *self == Self::ExecuteOnly
    }

    pub fn is_exec_only(&self) -> bool {
        *self == Self::ExecuteOnly
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Mode {
    #[default]
    Executable,
    Literal,
}

impl Mode {
    pub fn is_literal(self) -> bool {
        matches!(self, Mode::Literal)
    }

    pub fn is_executable(self) -> bool {
        matches!(self, Mode::Executable)
    }
}

#[derive(Debug)]
struct GState {}

#[derive(Debug)]
struct PackedArray {
    inner: Vec<u8>,
    mode: Mode,
}

#[derive(Debug)]
struct Save {}
