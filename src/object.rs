use std::{collections, fs, hash};

#[derive(Debug)]
pub enum Object<'a> {
    Integer(i32),
    Real(f64),
    Boolean(bool),
    Array(&'a Vec<Object<'a>>),
    PackedArray(&'a Vec<Object<'a>>),
    String(String),
    Name(String),
    Dictionary(&'a collections::HashMap<Object<'a>, Object<'a>>),
    Operator(String),
    File(fs::File),
    Mark,
    Null,
    Save,
    FontId, // TODO: Figure out what these things are
    GState,
}

impl hash::Hash for Object<'_> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Real(value) => f64::to_bits(*value).hash(state),
            Self::Dictionary(value) => {
                for key in value.keys() {
                    key.hash(state);
                }
            }
            Self::File(_) => "Object::File".hash(state),
            Self::Mark => "Object::Mark".hash(state),
            Self::Null => "Object::Null".hash(state),
            Self::Save => "Object::Save".hash(state),
            Self::FontId => "Object::FontId".hash(state),
            Self::GState => "Object::GState".hash(state),
            Self::Integer(value) => value.hash(state),
            Self::Boolean(value) => value.hash(state),
            Self::Array(value) => value.hash(state),
            Self::PackedArray(value) => value.hash(state),
            Self::String(value) => value.hash(state),
            Self::Name(value) => value.hash(state),
            Self::Operator(value) => value.hash(state),
        }
    }
}

impl PartialEq for Object<'_> {
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
            Self::Boolean(value) => {
                if let Self::Boolean(other_value) = other {
                    value == other_value
                } else {
                    false
                }
            }
            Self::Array(value) => {
                if let Self::Array(other_value) = other {
                    value.as_ptr() == other_value.as_ptr()
                } else {
                    false
                }
            }
            Self::PackedArray(value) => {
                if let Self::PackedArray(other_value) = other {
                    value.as_ptr() == other_value.as_ptr()
                } else {
                    false
                }
            }
            Self::String(value) => {
                if let Self::String(other_value) = other {
                    value == other_value
                } else {
                    false
                }
            }
            Self::Name(value) => {
                if let Self::Name(other_value) = other {
                    value == other_value
                } else {
                    false
                }
            }
            _ => return false,
        }
    }
}

impl Eq for Object<'_> {}
