use std::collections::HashMap;

use crate::{
    object::{Access, PostScriptDictionary},
    Error, ErrorKind, Interpreter, Object,
};

use super::usize_to_i32;

pub fn dict(interpreter: &mut Interpreter) -> crate::Result<()> {
    let capacity = interpreter.pop_usize()?;

    let idx = interpreter
        .dicts
        .insert(PostScriptDictionary::new(capacity, Access::default()));

    interpreter.push(Object::Dictionary(idx));

    Ok(())
}

pub fn enddict(interpreter: &mut Interpreter) -> crate::Result<()> {
    let mut dict = HashMap::new();

    loop {
        let value = interpreter.pop()?;
        if value.is_mark() {
            break;
        }

        let key = interpreter.pop_literal()?;
        if key.is_mark() {
            return Err(Error::from(ErrorKind::RangeCheck));
        }

        let key = key.to_string(interpreter)?;

        dict.insert(key, value);
    }

    let idx = interpreter.dicts.insert(dict.into());
    interpreter.push(Object::Dictionary(idx));

    Ok(())
}

pub fn maxlength(interpreter: &mut Interpreter) -> crate::Result<()> {
    let dict = interpreter.pop_dict()?;

    if !dict.access().is_readable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    let capacity = usize_to_i32(dict.capacity())?;

    interpreter.push(Object::Integer(capacity));

    Ok(())
}

pub fn begin(interpreter: &mut Interpreter) -> crate::Result<()> {
    let obj = interpreter.pop()?;

    match obj {
        Object::Dictionary(idx) => {
            interpreter.dict_stack.push(idx);
            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
    }
}

pub fn end(interpreter: &mut Interpreter) -> crate::Result<()> {
    if interpreter.dict_stack.len() <= 3 {
        return Err(Error::from(ErrorKind::DictStackUnderflow));
    }

    let _ = interpreter.dict_stack.pop();

    Ok(())
}

pub fn def(interpreter: &mut Interpreter) -> crate::Result<()> {
    let value = interpreter.pop()?;
    let key = interpreter.pop_literal()?.to_string(interpreter)?;

    let dict_idx = interpreter
        .dict_stack
        .last()
        .ok_or(Error::from(ErrorKind::LimitCheck))?;

    let dict = interpreter.dicts.get_mut(*dict_idx)?;

    if !dict.access().is_writeable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    dict.insert(key, value)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error;

    #[test]
    fn test_dict() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_enddict() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_maxlength() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_begin() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_end() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn test_def() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }
}
