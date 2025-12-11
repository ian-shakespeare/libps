use std::hash::Hash;

use crate::{composite::Composite, name::NameObject, operator::OperatorObject, Error, ErrorKind};

#[derive(Clone)]
pub enum Object {
    // Simple
    Boolean(bool),
    FontID,
    Integer(i32),
    Mark,
    Name(NameObject),
    Null(Mode),
    Operator(OperatorObject),
    Real(f32),

    // Composite
    Array(Composite),
    Dictionary(Composite),
    File(Composite),
    GState(Composite),
    PackedArray(Composite),
    Save(Composite),
    String(Composite),
}

impl Object {
    pub fn into_composite(self) -> crate::Result<Composite> {
        match self {
            Object::Array(cmp)
            | Object::Dictionary(cmp)
            | Object::File(cmp)
            | Object::GState(cmp)
            | Object::PackedArray(cmp)
            | Object::Save(cmp)
            | Object::String(cmp) => Ok(cmp),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected composite")),
        }
    }

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
            Object::Operator(operator) => operator.mode,
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
            Object::Operator(operator) => operator.mode = mode,
            Object::Array(array) => array.mode = mode,
            Object::File(file) => file.mode = mode,
            Object::String(string) => string.mode = mode,
            _ => (),
        };
    }
}

impl Eq for Object {}

impl From<Error> for Object {
    fn from(value: Error) -> Self {
        let mut name = NameObject::from(match value.kind() {
            ErrorKind::DictStackUnderflow => "dictstackunderflow",
            ErrorKind::InvalidAccess => "invalidaccess",
            ErrorKind::IoError => "ioerror",
            ErrorKind::LimitCheck => "limitcheck",
            ErrorKind::RangeCheck => "rangecheck",
            ErrorKind::StackUnderflow => "stackunderflow",
            ErrorKind::SyntaxError => "syntaxerror",
            ErrorKind::TypeCheck => "typecheck",
            ErrorKind::Undefined => "undefined",
            ErrorKind::UndefinedResult => "undefinedresult",
            ErrorKind::UnmatchedMark => "unmatchedmark",
            ErrorKind::Unregistered => "unregistered",
            ErrorKind::VmError => "vmerror",
        });
        name.mode = Mode::Executable;

        Self::Name(name)
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::Boolean(b) => b.hash(state),
            Object::FontID => todo!(),
            Object::Integer(i) => i.hash(state),
            Object::Mark => todo!(),
            Object::Name(n) => n.hash(state),
            Object::Null(_) => todo!(),
            Object::Operator(o) => o.hash(state),
            Object::Real(r) => r.to_bits().hash(state),
            Object::Array(a) => a.hash(state),
            Object::Dictionary(d) => d.hash(state),
            Object::File(f) => f.hash(state),
            Object::GState(g) => g.hash(state),
            Object::PackedArray(pa) => pa.hash(state),
            Object::Save(s) => s.hash(state),
            Object::String(s) => s.hash(state),
        }
    }
}

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
            (Object::String(lhs), Object::String(rhs)) => todo!(),
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
