use std::collections::HashMap;

use crate::{Error, ErrorKind, Interpreter, Object};

use super::usize_to_i32;

pub fn dict(interpreter: &mut Interpreter) -> crate::Result<()> {
    let capacity = interpreter.pop_usize()?;

    let idx = interpreter.mem.insert(HashMap::with_capacity(capacity));

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

    let idx = interpreter.mem.insert(dict);
    interpreter.push(Object::Dictionary(idx));

    Ok(())
}

pub fn maxlength(interpreter: &mut Interpreter) -> crate::Result<()> {
    let dict = interpreter.pop_dict()?;

    if !dict.access.is_readable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    let capacity = dict.len() + 1;

    let capacity = usize_to_i32(capacity)?;

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

    let dict = interpreter.mem.get_mut(*dict_idx)?;

    if !dict.access.is_writeable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    dict.dict_mut()?.insert(key, value);

    Ok(())
}

pub fn load(interpreter: &mut Interpreter) -> crate::Result<()> {
    let key_obj = interpreter.pop_literal()?;
    let key = interpreter.stringify(&key_obj)?;

    let obj = interpreter.find(key)?;

    interpreter.push(obj.clone());

    Ok(())
}

pub fn store(interpreter: &mut Interpreter) -> crate::Result<()> {
    let value = interpreter.pop()?;
    let key_obj = interpreter.pop_literal()?;
    let key = interpreter.stringify(&key_obj)?;

    let obj = interpreter.find_mut(key)?;
    *obj = value;

    Ok(())
}

pub fn undef(interpreter: &mut Interpreter) -> crate::Result<()> {
    let key_obj = interpreter.pop_literal()?;
    let key = interpreter.stringify(&key_obj)?;
    let dict = interpreter.pop_dict_mut()?;

    if !dict.access.is_writeable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    match dict.dict_mut()?.remove(&key) {
        Some(..) => Ok(()),
        None => Err(Error::new(ErrorKind::Undefined, key)),
    }
}

pub fn known(interpreter: &mut Interpreter) -> crate::Result<()> {
    let key_obj = interpreter.pop_literal()?;
    let key = interpreter.stringify(&key_obj)?;
    let dict = interpreter.pop_dict()?;

    if !dict.access.is_readable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    let exists = dict.dict()?.contains_key(&key);

    interpreter.push(Object::Boolean(exists));

    Ok(())
}

pub fn wheredef(interpreter: &mut Interpreter) -> crate::Result<()> {
    let key_obj = interpreter.pop_literal()?;
    let key = interpreter.stringify(&key_obj)?;

    match interpreter.find_dict(key) {
        Ok(idx) => {
            interpreter.push(Object::Dictionary(idx));
            interpreter.push(Object::Boolean(true));
        },
        Err(..) => {
            interpreter.push(Object::Boolean(false));
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error;

    use crate::composite::Composite;

    use super::*;

    #[test]
    fn test_dict() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.push(Object::Integer(5));
        dict(&mut interpreter)?;

        assert_eq!(1, interpreter.operand_stack.len());

        let dict = interpreter.pop_dict()?;
        assert_eq!(0, dict.len());

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
        assert_eq!(0, dict.dict()?.capacity());

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
        let mut interpreter = Interpreter::default();
        let idx = interpreter.mem.insert(HashMap::new());
        interpreter.push(Object::Dictionary(idx));
        maxlength(&mut interpreter)?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(1, interpreter.pop_int()?);

        Ok(())
    }

    #[test]
    fn test_maxlength_underflow() {
        let mut interpreter = Interpreter::default();

        let result = maxlength(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_begin() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        let idx = interpreter.mem.insert(HashMap::new());
        interpreter.push(Object::Dictionary(idx));
        begin(&mut interpreter)?;

        assert_eq!(0, interpreter.operand_stack.len());
        assert_eq!(4, interpreter.dict_stack.len());
        assert_eq!(Some(idx), interpreter.dict_stack.last().cloned());

        Ok(())
    }

    #[test]
    fn test_begin_typecheck() {
        let mut interpreter = Interpreter::default();
        interpreter.push(Object::Integer(0));

        let result = begin(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_begin_underflow() {
        let mut interpreter = Interpreter::default();

        let result = begin(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_end() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        let idx = interpreter.mem.insert(HashMap::new());
        interpreter.dict_stack.push(idx);
        end(&mut interpreter)?;

        assert_eq!(3, interpreter.dict_stack.len());

        Ok(())
    }

    #[test]
    fn test_end_dictunderflow() {
        let mut interpreter = Interpreter::default();

        let result = end(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::DictStackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_def() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.push(Object::Name("/pi".into()));
        interpreter.push(Object::Real(3.1415926));
        def(&mut interpreter)?;

        assert_eq!(0, interpreter.operand_stack.len());

        let idx = interpreter.dict_stack.last().ok_or("expected dict")?;
        let userdict = interpreter.mem.get_dict(*idx)?;
        assert!(userdict.contains_key("/pi"));
        assert_eq!(Some(Object::Real(3.1415926)), userdict.get("/pi").cloned());

        Ok(())
    }

    #[test]
    fn test_def_invalidaccess() {
        assert!(false, "access levels are not yet defined");
    }

    #[test]
    fn test_def_underflow() {
        let mut interpreter = Interpreter::default();
        interpreter.push(Object::Integer(0));

        let result = def(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_load() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        let mut dict = HashMap::new();
        dict.insert("/key".to_string(), Object::Integer(1));

        let idx = interpreter.mem.insert(dict);
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

        let idx = interpreter.mem.insert(dict);
        interpreter.dict_stack.push(idx);

        interpreter.push(Object::Name("/key".to_string()));
        interpreter.push(Object::Integer(2));
        store(&mut interpreter)?;

        assert_eq!(0, interpreter.operand_stack.len());

        let val = interpreter
            .mem
            .get_dict(idx)?
            .get("/key")
            .ok_or("expected value")?;
        assert_eq!(Object::Integer(2), val.clone());

        Ok(())
    }

    // TODO: Check for perms when loading and storing

    #[test]
    fn test_undef() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        let mut dict = HashMap::new();
        dict.insert("/key".to_string(), Object::Integer(1));

        let idx = interpreter.mem.insert(dict);
        interpreter.operand_stack.push(Object::Dictionary(idx));
        interpreter
            .operand_stack
            .push(Object::Name("/key".to_string()));

        undef(&mut interpreter)?;
        assert_eq!(0, interpreter.operand_stack.len());

        let dict = interpreter.mem.get_dict(idx)?;
        assert_eq!(None, dict.get("/key"));

        Ok(())
    }

    #[test]
    fn test_undef_undefined() {
        let mut interpreter = Interpreter::default();
        let mut dict = HashMap::new();
        dict.insert("/key".to_string(), Object::Integer(1));

        let idx = interpreter.mem.insert(dict);
        interpreter.operand_stack.push(Object::Dictionary(idx));
        interpreter
            .operand_stack
            .push(Object::Name("/otherKey".to_string()));

        let result = undef(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::Undefined, result.unwrap_err().kind());
    }

    #[test]
    fn test_undef_underflow() {
        let mut interpreter = Interpreter::default();
        interpreter
            .operand_stack
            .push(Object::Name("/otherKey".to_string()));

        let result = undef(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_known() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        let mut dict = HashMap::new();
        dict.insert("/key".to_string(), Object::Integer(1));

        let idx = interpreter.mem.insert(dict);
        interpreter.operand_stack.push(Object::Dictionary(idx));
        interpreter
            .operand_stack
            .push(Object::Name("/key".to_string()));

        known(&mut interpreter)?;
        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(Object::Boolean(true), interpreter.pop()?);

        interpreter.operand_stack.push(Object::Dictionary(idx));
        interpreter
            .operand_stack
            .push(Object::Name("/otherKey".to_string()));

        known(&mut interpreter)?;
        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(Object::Boolean(false), interpreter.pop()?);

        Ok(())
    }

    #[test]
    fn test_known_underflow() {
        let mut interpreter = Interpreter::default();
        interpreter
            .operand_stack
            .push(Object::Name("/otherKey".to_string()));

        let result = known(&mut interpreter);
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_where() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        let mut dict = HashMap::new();
        dict.insert("/key".to_string(), Object::Integer(1));

        let idx = interpreter.mem.insert(dict);
        interpreter.dict_stack.push(idx);
        interpreter
            .operand_stack
            .push(Object::Name("/key".to_string()));

        wheredef(&mut interpreter)?;
        assert_eq!(2, interpreter.operand_stack.len());
        assert_eq!(Object::Boolean(true), interpreter.pop()?);
        assert!(matches!(interpreter.pop()?, Object::Dictionary(..)));

        interpreter
            .operand_stack
            .push(Object::Name("/otherKey".to_string()));

        wheredef(&mut interpreter)?;
        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(Object::Boolean(false), interpreter.pop()?);

        Ok(())
    }
}
