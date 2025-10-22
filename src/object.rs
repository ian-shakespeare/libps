use std::marker::PhantomData;

use crate::{
    array::ArrayObject, dictionary::DictionaryObject, file::FileObject, memory::Address,
    name::NameObject, operator::OperatorObject, string::StringObject, Error, ErrorKind,
};

pub struct Composite<T> {
    pub address: Address,
    pub access: Access,
    pub mode: Mode,
    pub _marker: PhantomData<T>,
}

impl<T> Composite<T> {
    pub fn address(&self) -> Address {
        self.address
    }
}

impl<T> Clone for Composite<T> {
    fn clone(&self) -> Self {
        Self {
            address: self.address,
            access: self.access,
            mode: self.mode,
            _marker: self._marker,
        }
    }
}

impl<T> PartialEq for Composite<T> {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode && self.address == other.address
    }
}

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
    Array(Composite<ArrayObject>),
    Dictionary(Composite<DictionaryObject>),
    File(Composite<FileObject>),
    GState(Composite<GState>),
    PackedArray(Composite<PackedArray>),
    Save(Composite<Save>),
    String(Composite<StringObject>),
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
            Object::Name(name) => name.mode,
            Object::Null(mode) => *mode,
            Object::Operator((_, mode)) => *mode,
            Object::Array(array) => array.mode,
            Object::File(file) => file.mode,
            Object::String(string) => string.mode,
            _ => Mode::Literal,
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        match self {
            Object::Name(name) => name.mode = mode,
            Object::Null(current) => *current = mode,
            Object::Operator((_, current)) => *current = mode,
            Object::Array(array) => array.mode = mode,
            Object::File(file) => file.mode = mode,
            Object::String(string) => string.mode = mode,
            _ => (),
        };
    }
}

impl Eq for Object {}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Boolean(lhs), Object::Boolean(rhs)) => lhs == rhs,
            (Object::Integer(lhs), Object::Integer(rhs)) => lhs == rhs,
            (Object::Integer(lhs), Object::Real(rhs)) => *lhs as f32 == *rhs,
            (Object::Name(lhs), Object::Name(rhs)) => lhs == rhs,
            (Object::Name(lhs), Object::String(rhs)) => todo!(),
            (Object::Operator(lhs), Object::Operator(rhs)) => lhs == rhs,
            (Object::Real(lhs), Object::Real(rhs)) => lhs == rhs,
            (Object::Real(lhs), Object::Integer(rhs)) => *lhs == *rhs as f32,
            (Object::Array(lhs), Object::Array(rhs)) => lhs == rhs,
            (Object::Dictionary(lhs), Object::Dictionary(rhs)) => lhs == rhs,
            (Object::File(lhs), Object::File(rhs)) => lhs == rhs,
            (Object::String(lhs), Object::String(rhs)) => lhs == rhs,
            (Object::String(lhs), Object::Name(rhs)) => todo!(),
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
