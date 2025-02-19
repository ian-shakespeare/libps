use crate::{
    object::{Access, PostScriptArray},
    Error, ErrorKind, Interpreter, Object,
};

use super::usize_to_i32;

pub fn array(interpreter: &mut Interpreter) -> crate::Result<()> {
    let len = interpreter.pop_usize()?;

    let idx = interpreter.arrays.insert(vec![Object::Null; len].into());

    interpreter.push(Object::Array(idx));

    Ok(())
}

pub fn endarray(interpreter: &mut Interpreter) -> crate::Result<()> {
    let mut arr = Vec::new();

    loop {
        match interpreter.pop() {
            Err(_) => return Err(Error::from(ErrorKind::UnmatchedMark)),
            Ok(obj) => {
                if matches!(obj, Object::Mark) {
                    break;
                }

                arr.push(obj);
            },
        }
    }

    arr.reverse();
    let idx = interpreter.arrays.insert(arr.into());

    interpreter.push(Object::Array(idx));

    Ok(())
}

pub fn length(interpreter: &mut Interpreter) -> crate::Result<()> {
    let obj = interpreter.pop()?;

    let len = match obj {
        Object::Array(idx) | Object::PackedArray(idx) => {
            let arr = interpreter.arrays.get(idx)?;

            if !arr.access().is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            Ok(arr.len())
        },
        Object::Dictionary(idx) => {
            let dict = interpreter.dicts.get(idx)?;

            if !dict.access().is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            Ok(dict.len())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    let i = usize_to_i32(len)?;
    interpreter.push(Object::Integer(i));

    Ok(())
}

pub fn get(interpreter: &mut Interpreter) -> crate::Result<()> {
    let key = interpreter.pop_literal()?;
    let obj = interpreter.pop()?;

    let obj = match obj {
        Object::Array(idx) | Object::PackedArray(idx) => {
            let arr = interpreter.arrays.get(idx)?;

            if arr.access().is_exec_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let index = key.into_usize()?;

            match arr.value().get(index) {
                Some(obj) => Ok(obj.clone()),
                None => Err(Error::from(ErrorKind::RangeCheck)),
            }
        },
        Object::Dictionary(idx) => {
            let dict = interpreter.dicts.get(idx)?;

            if !dict.access().is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let key = interpreter.stringify(&key)?;

            match dict.value().get(&key) {
                Some(obj) => Ok(obj.clone()),
                None => Err(Error::new(ErrorKind::Undefined, key)),
            }
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    interpreter.push(obj);

    Ok(())
}

pub fn put(interpreter: &mut Interpreter) -> crate::Result<()> {
    let value = interpreter.pop()?;
    let key = interpreter.pop_literal()?;
    let obj = interpreter.pop()?;

    match obj {
        Object::Array(idx) => {
            let arr = interpreter.arrays.get_mut(idx)?;

            if !arr.access().is_writeable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let index = key.into_usize()?;

            let obj = arr
                .value_mut()
                .get_mut(index)
                .ok_or(Error::from(ErrorKind::RangeCheck))?;
            *obj = value;

            Ok(())
        },
        Object::Dictionary(idx) => {
            let key = interpreter.stringify(&key)?;
            let dict = interpreter.dicts.get_mut(idx)?;

            if !dict.access().is_writeable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let obj = dict.get_mut(key)?;
            *obj = value;

            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }
}

pub fn getinterval(interpreter: &mut Interpreter) -> crate::Result<()> {
    let count = interpreter.pop_usize()?;
    let index = interpreter.pop_usize()?;
    let obj = interpreter.pop()?;

    let arr = match obj {
        Object::Array(idx) | Object::PackedArray(idx) => {
            let arr = interpreter.arrays.get(idx)?;

            if !arr.access().is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            Ok(arr.value().clone())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    if index >= arr.len() {
        return Err(Error::from(ErrorKind::RangeCheck));
    }

    let mut subarr = Vec::with_capacity(count);

    for i in index..(index + count) {
        let obj = arr.get(i).ok_or(Error::from(ErrorKind::RangeCheck))?;

        subarr.push(obj.clone());
    }

    let idx = interpreter.arrays.insert(subarr.into());

    interpreter.push(Object::Array(idx));

    Ok(())
}

pub fn putinterval(interpreter: &mut Interpreter) -> crate::Result<()> {
    let source = interpreter.pop_array()?;

    if !source.access().is_readable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    let source = source.value().clone();

    let index = interpreter.pop_usize()?;
    let destination = interpreter.pop_array_mut()?;

    if !destination.access().is_writeable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    for (offset, obj) in source.into_iter().enumerate() {
        let Some(dest_obj) = destination.value_mut().get_mut(index + offset) else {
            return Err(Error::from(ErrorKind::RangeCheck));
        };

        *dest_obj = obj;
    }

    Ok(())
}

pub fn astore(interpreter: &mut Interpreter) -> crate::Result<()> {
    let Object::Array(arr_idx) = interpreter.pop()? else {
        return Err(Error::new(ErrorKind::TypeCheck, "expected array"));
    };

    let len = match interpreter.arrays.get(arr_idx) {
        Ok(composite) => {
            if !composite.access().is_writeable() {
                Err(Error::from(ErrorKind::InvalidAccess))
            } else {
                Ok(composite.len())
            }
        },
        Err(_) => Err(Error::from(ErrorKind::Undefined)),
    }?;

    let mut stored = vec![Object::Null; len];

    for i in (0..len).rev() {
        stored[i] = interpreter.pop()?;
    }

    match interpreter.arrays.get_mut(arr_idx) {
        Ok(composite) => {
            let existing = composite.value_mut();
            *existing = stored;
            Ok(())
        },
        Err(_) => Err(Error::from(ErrorKind::Undefined)),
    }
}

pub fn aload(interpreter: &mut Interpreter) -> crate::Result<()> {
    let obj = interpreter.pop()?;

    let (idx, arr) = match obj {
        Object::Array(idx) | Object::PackedArray(idx) => {
            let arr = interpreter.arrays.get(idx)?;

            if !arr.access().is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            Ok((idx, arr.value().clone()))
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    for obj in arr {
        interpreter.push(obj);
    }

    interpreter.push(Object::Array(idx));

    Ok(())
}

pub fn forall(interpreter: &mut Interpreter) -> crate::Result<()> {
    let proc = interpreter.pop()?;
    let obj = interpreter.pop()?;

    if !proc.is_procedure() {
        return Err(Error::new(ErrorKind::TypeCheck, "expected procedure"));
    }

    match obj {
        Object::Array(idx) => {
            let arr = interpreter.arrays.get(idx)?;

            for obj in arr.value().clone() {
                interpreter.push(obj);
                interpreter.execute_object(proc.clone())?;
            }

            Ok(())
        },
        Object::Dictionary(idx) => {
            let dict = interpreter.dicts.get(idx)?;

            for (key, value) in dict.value().clone() {
                interpreter.push(Object::Name(format!("/{key}")));
                interpreter.push(value);

                interpreter.execute_object(proc.clone())?;
            }

            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }
}

pub fn packedarray(interpreter: &mut Interpreter) -> crate::Result<()> {
    let size = interpreter.pop_usize()?;

    let arr = PostScriptArray::new(vec![Object::Null; size], Access::ReadOnly);

    let index = interpreter.arrays.insert(arr);

    interpreter.push(Object::PackedArray(index));

    Ok(())
}

pub fn setpacking(interpreter: &mut Interpreter) -> crate::Result<()> {
    let b = interpreter.pop_bool()?;

    interpreter.is_packing = b;

    Ok(())
}

pub fn currentpacking(interpreter: &mut Interpreter) -> crate::Result<()> {
    interpreter.push(Object::Boolean(interpreter.is_packing));

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, error};

    use super::*;

    #[test]
    fn test_array() -> Result<(), Box<dyn error::Error>> {
        for num in 0..5 {
            let input = format!("{num} array");
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            let Some(Object::Array(arr_idx)) = interpreter.operand_stack.pop() else {
                return Err("expected array object".into());
            };

            let arr = interpreter.arrays.get(arr_idx)?;
            assert_eq!(num, arr.len());
            assert!(arr.value().iter().all(|obj| matches!(obj, Object::Null)));
        }

        Ok(())
    }

    #[test]
    fn test_array_rangecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("-1 array".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind())
    }

    #[test]
    fn test_array_typeheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("(str) array".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind())
    }

    #[test]
    fn test_array_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("array".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind())
    }

    #[test]
    fn test_startarray() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());

        let obj = interpreter.operand_stack.pop().ok_or("expected object")?;
        assert!(matches!(obj, Object::Mark));

        Ok(())
    }

    #[test]
    fn test_endarray() -> Result<(), Box<dyn error::Error>> {
        for length in 0..5 {
            let input = "[ ".to_string() + &format!("0 ").repeat(length) + "]";

            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            let Some(Object::Array(arr_idx)) = interpreter.operand_stack.pop() else {
                return Err("expected array object".into());
            };

            let arr = interpreter.arrays.get(arr_idx)?;
            assert_eq!(length, arr.len());
            assert!(arr
                .value()
                .iter()
                .all(|obj| matches!(obj, Object::Integer(0))));
        }

        Ok(())
    }

    #[test]
    fn test_endarray_unmatchedmark() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("]".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::UnmatchedMark, result.unwrap_err().kind());
    }

    #[test]
    fn test_length() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 4 5 ] length".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(5, interpreter.pop_int()?);

        Ok(())
    }

    #[test]
    fn test_length_packedarray() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("5 packedarray length".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(5, interpreter.pop_int()?);

        Ok(())
    }

    #[test]
    fn test_get() -> Result<(), Box<dyn error::Error>> {
        for i in 0..5 {
            let input = format!("[ 1 2 3 4 5 ] {i} get");
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            let received = interpreter.pop_int()?;

            assert_eq!(i + 1, received);
        }

        Ok(())
    }

    #[test]
    fn test_get_packedarray() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 packedarray 0 get".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert!(matches!(interpreter.pop()?, Object::Null));

        Ok(())
    }

    #[test]
    fn test_get_dictionary() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("<</key 1>> /key get".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(Object::Integer(1), interpreter.pop()?);

        Ok(())
    }

    #[test]
    fn test_get_rangecheck() {
        let inputs = ["[ 1 2 3 ] -1 get", "[ 1 2 3 ] 3 get", "[ ] 0 get"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_get_typecheck() {
        let inputs = ["[ 1 2 3 ] (str) get", "1 3 get"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_get_undefined() {
        let mut interpreter = Interpreter::default();

        let result = interpreter.evaluate("<</key 1>> /otherKey get".chars().into());
        assert!(result.is_err());
        assert_eq!(ErrorKind::Undefined, result.unwrap_err().kind());
    }

    #[test]
    fn test_get_underflow() {
        let inputs = ["0 get", "get"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_put() -> Result<(), Box<dyn error::Error>> {
        for i in 0..5 {
            let input = format!("5 array {i} 3.14 put");
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(0, interpreter.operand_stack.len());

            let arr = interpreter.arrays.get(1)?;
            assert_eq!(Some(Object::Real(3.14)), arr.value().get(i).cloned());
        }

        Ok(())
    }

    #[test]
    fn test_put_dictionary() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        let mut dict = HashMap::new();

        dict.insert("key".to_string(), Object::Integer(1));
        let dict_idx = interpreter.dicts.insert(dict.into());
        interpreter.operand_stack.push(Object::Dictionary(dict_idx));

        interpreter.evaluate("/key 2 put".chars().into())?;

        assert_eq!(0, interpreter.operand_stack.len());

        let dict = interpreter.dicts.get(dict_idx)?;
        assert_eq!(Some(Object::Integer(2)), dict.value().get("key").cloned());

        Ok(())
    }

    #[test]
    fn test_put_rangecheck() {
        let inputs = [
            "[ 1 2 3 ] -1 3.14 put",
            "[ 1 2 3 ] 3 3.14 put",
            "[ ] 0 3.14 put",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_put_typecheck() {
        let inputs = ["[ 1 2 3 ] (str) 3.14 put", "0 3 3.14 put"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_put_undefined() {
        let mut interpreter = Interpreter::default();

        let result = interpreter.evaluate("<</key 1>> /otherKey 2 put".chars().into());
        assert!(result.is_err());
        assert_eq!(ErrorKind::Undefined, result.unwrap_err().kind());
    }

    #[test]
    fn test_put_underflow() {
        let inputs = ["0 3.14 put", "3.14 put", "put"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_getinterval() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 4 5 ] 1 3 getinterval".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());

        let arr = interpreter.pop_array()?;
        assert_eq!(
            vec![Object::Integer(2), Object::Integer(3), Object::Integer(4)],
            arr.value().clone()
        );

        Ok(())
    }

    #[test]
    fn test_getinterval_rangecheck() {
        let inputs = [
            "[ 1 2 3 ] -1 0 getinterval",
            "[ 1 2 3 ] 0 -1 getinterval",
            "[ 1 2 3 ] 1000 0 getinterval",
            "[ 1 2 3 ] 0 1000 getinterval",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_getinterval_typecheck() {
        let inputs = [
            "3.14 0 1 getinterval",
            "[ 1 ] (str) 1 getinterval",
            "[ 1 ] 0 (str) getinterval",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_getinterval_underflow() {
        let inputs = ["0 1 getinterval", "1 getinterval", "getinterval"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_putinterval() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 4 5 ] 1 [ 6 7 8 ] putinterval".chars().into())?;

        let arr = interpreter.arrays.get(1)?;
        assert_eq!(
            vec![
                Object::Integer(1),
                Object::Integer(6),
                Object::Integer(7),
                Object::Integer(8),
                Object::Integer(5),
            ],
            arr.value().clone()
        );

        Ok(())
    }

    #[test]
    fn test_putinterval_rangecheck() {
        let inputs = [
            "[ 1 2 3 ] 1000 [ 4 ] putinterval",
            "[ 1 2 3 ] -1 [ 4 ] putinterval",
            "[ 1 2 3 ] 2 [ 4 5 ] putinterval",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_putinterval_typecheck() {
        let inputs = [
            "3.14 0 [ 4 ] putinterval",
            "[ 1 2 3 ] (str) [ 4 ] putinterval",
            "[ 1 2 3 ] 0 3.14 putinterval",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_putinterval_underflow() {
        let inputs = ["0 [ 4 ] putinterval", "[ 4 ] putinterval", "putinterval"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_astore() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 3 4 5 3 array astore".chars().into())?;

        assert_eq!(2, interpreter.operand_stack.len());

        let arr = interpreter.arrays.get(1)?;
        assert_eq!(
            vec![Object::Integer(3), Object::Integer(4), Object::Integer(5)],
            arr.value().clone()
        );

        Ok(())
    }

    #[test]
    fn test_astore_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 2 3 3.14 astore".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_astore_underflow() {
        let inputs = ["1 2 3 10 array astore", "astore"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_aload() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 4 5 ] aload".chars().into())?;

        assert_eq!(6, interpreter.operand_stack.len());

        let arr = interpreter.pop_array()?;
        assert_eq!(
            vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
                Object::Integer(5)
            ],
            arr.value().clone()
        );

        assert_eq!(5, interpreter.pop_int()?);
        assert_eq!(4, interpreter.pop_int()?);
        assert_eq!(3, interpreter.pop_int()?);
        assert_eq!(2, interpreter.pop_int()?);
        assert_eq!(1, interpreter.pop_int()?);

        Ok(())
    }

    #[test]
    fn test_aload_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 aload".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_aload_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("aload".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_forall() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("0 [ 13 29 3 -8 21 ] { add } forall".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(58)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_forall_dictionary() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("<</abc 1 /xyz 2>> {} forall".chars().into())?;

        assert_eq!(4, interpreter.operand_stack.len());
        assert!(matches!(interpreter.pop()?, Object::Integer(..)));
        assert!(matches!(interpreter.pop_literal()?, Object::Name(..)));
        assert!(matches!(interpreter.pop()?, Object::Integer(..)));
        assert!(matches!(interpreter.pop_literal()?, Object::Name(..)));

        Ok(())
    }

    #[test]
    fn test_packedarray() -> Result<(), Box<dyn error::Error>> {
        for num in 0..5 {
            let input = format!("{num} packedarray");
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            let Some(Object::PackedArray(arr_idx)) = interpreter.operand_stack.pop() else {
                return Err("expected packed array object".into());
            };

            let arr = interpreter.arrays.get(arr_idx)?;
            assert!(arr.access().is_read_only());
            assert_eq!(num, arr.len());
            assert!(arr.value().iter().all(|obj| matches!(obj, Object::Null)));
        }

        Ok(())
    }

    #[test]
    fn test_packedarray_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("(str) packedarray".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_packedarray_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("packedarray".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_setpacking() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("true setpacking".chars().into())?;

        assert!(interpreter.is_packing);

        Ok(())
    }

    #[test]
    fn test_setpacking_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("(str) setpacking".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_setpacking_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("setpacking".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_currentpacking() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("true setpacking currentpacking".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert!(interpreter.pop_bool()?);

        Ok(())
    }
}
