use std::{collections, rc};

#[derive(Clone, Debug)]
pub enum Object {
    Integer(i32),
    Real(f64),
    Boolean(bool),
    Array(rc::Rc<Vec<Object>>),
    PackedArray(rc::Rc<Vec<Object>>),
    String(String),
    LiteralName(String),
    Name(String),
    Dictionary(rc::Rc<collections::HashMap<String, Object>>),
    // File(Box<fs::File>),
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
            Object::String(value) | Object::Name(value) => value,
            Object::Array(values) => {
                let mut output = String::from('[');
                for obj in values.iter() {
                    output.push_str(&format!(" {}", &String::from(obj.clone())))
                }
                output.push_str(" ]");
                output
            }
            Object::PackedArray(values) => {
                let mut output = String::from('{');
                for obj in values.iter() {
                    output.push_str(&format!(" {}", &String::from(obj.clone())))
                }
                output.push_str(" }");
                output
            }
            Object::Dictionary(values) => {
                let mut output = String::from("<<");
                for (key, value) in values.iter() {
                    output.push_str(&format!(" {} {}", &key, &String::from(value.clone())));
                }
                output.push_str(" >>");
                output
            }
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
            _ => false,
        }
    }
}

impl Eq for Object {}
