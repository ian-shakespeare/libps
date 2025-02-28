use std::collections::HashMap;

use crate::{Error, ErrorKind, Object};

pub enum Value {
    Array(Vec<Object>),
    Dictionary(HashMap<String, Object>),
    PackedArray(Vec<Object>),
    String(String),
}

impl From<Vec<Object>> for Value {
    fn from(value: Vec<Object>) -> Self {
        Self::Array(value)
    }
}

impl<'a> TryFrom<&'a Value> for &'a Vec<Object> {
    type Error = crate::Error;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(ref a) => Ok(a),
            Value::PackedArray(ref p) => Ok(p),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }
}

impl<'a> TryFrom<&'a mut Value> for &'a mut Vec<Object> {
    type Error = crate::Error;

    fn try_from(value: &'a mut Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(ref mut a) => Ok(a),
            Value::PackedArray(ref mut p) => Ok(p),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }
}

impl From<HashMap<String, Object>> for Value {
    fn from(value: HashMap<String, Object>) -> Self {
        Self::Dictionary(value)
    }
}

impl<'a> TryFrom<&'a Value> for &'a HashMap<String, Object> {
    type Error = crate::Error;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Dictionary(ref d) => Ok(d),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }
}

impl<'a> TryFrom<&'a mut Value> for &'a mut HashMap<String, Object> {
    type Error = crate::Error;

    fn try_from(value: &'a mut Value) -> Result<Self, Self::Error> {
        match value {
            Value::Dictionary(ref mut d) => Ok(d),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = crate::Error;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(ref s) => Ok(s),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }
}
