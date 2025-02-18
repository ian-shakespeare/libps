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

        let key = interpreter.stringify(&key)?;

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
    let key_obj = interpreter.pop_literal()?;
    let key = interpreter.stringify(&key_obj)?;

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

pub fn load(interpreter: &mut Interpreter) -> crate::Result<()> {
    let key_obj = interpreter.pop_literal()?;
    let key = interpreter.stringify(&key_obj)?;

    let obj = interpreter.search(key)?;

    interpreter.push(obj.clone());

    Ok(())
}

pub fn store(interpreter: &mut Interpreter) -> crate::Result<()> {
    let value = interpreter.pop()?;
    let key_obj = interpreter.pop_literal()?;
    let key = interpreter.stringify(&key_obj)?;

    let obj = interpreter.search_mut(key)?;
    *obj = value;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    #[test]
    fn test_dict() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.push(Object::Integer(5));
        dict(&mut interpreter)?;

        assert_eq!(1, interpreter.operand_stack.len());

        let dict = interpreter.pop_dict()?;
        assert_eq!(0, dict.len());
        assert_eq!(5, dict.capacity());

        Ok(())
    }

    #[test]
    fn test_dict_rangecheck() {
        let mut interpreter = Interpreter::default();
        interpreter.push(Object::Integer(-1));

        let result = dict(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_dict_typecheck() {
        let mut interpreter = Interpreter::default();
        interpreter.push(Object::Boolean(true));

        let result = dict(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_dict_underflow() {
        let mut interpreter = Interpreter::default();

        let result = dict(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_enddict() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.push(Object::Mark);
        enddict(&mut interpreter)?;

        let dict = interpreter.pop_dict()?;
        assert_eq!(0, dict.len());
        assert_eq!(0, dict.capacity());

        Ok(())
    }

    #[test]
    fn test_enddict_unmatchedmark() {
        let mut interpreter = Interpreter::default();

        let result = enddict(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_maxlength() -> Result<(), Box<dyn error::Error>> {
        Err("not implemented".into())
    }

    #[test]
    fn test_begin() -> Result<(), Box<dyn error::Error>> {
        Err("not implemented".into())
    }

    #[test]
    fn test_end() -> Result<(), Box<dyn error::Error>> {
        Err("not implemented".into())
    }

    #[test]
    fn test_def() -> Result<(), Box<dyn error::Error>> {
        Err("not implemented".into())
    }

    #[test]
    fn test_load() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        let mut dict = HashMap::new();
        dict.insert("/key".to_string(), Object::Integer(1));

        let idx = interpreter.dicts.insert(dict.into());
        interpreter.dict_stack.push(idx);
        interpreter.push(Object::Name("/key".to_string()));
        load(&mut interpreter)?;

        assert_eq!(1, interpreter.operand_stack.len());

        let i = interpreter.pop_int()?;
        assert_eq!(1, i);

        Ok(())
    }

    #[test]
    fn test_store() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        let mut dict = HashMap::new();
        dict.insert("/key".to_string(), Object::Integer(1));

        let idx = interpreter.dicts.insert(dict.into());
        interpreter.dict_stack.push(idx);

        interpreter.push(Object::Name("/key".to_string()));
        interpreter.push(Object::Integer(2));
        store(&mut interpreter)?;

        assert_eq!(0, interpreter.operand_stack.len());

        let val = interpreter.dicts.get(idx)?.get("/key".to_string())?;
        assert_eq!(Object::Integer(2), val.clone());

        Ok(())
    }

    // TODO: Check for perms when loading and storing
}
