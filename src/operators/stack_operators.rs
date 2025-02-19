use crate::{Error, ErrorKind, Interpreter, Object};

use super::usize_to_i32;

pub fn dup(interpreter: &mut Interpreter) -> crate::Result<()> {
    let obj = interpreter.pop()?;

    interpreter.push(obj.clone());
    interpreter.push(obj);

    Ok(())
}

pub fn exch(interpreter: &mut Interpreter) -> crate::Result<()> {
    let first = interpreter.pop()?;
    let second = interpreter.pop()?;

    interpreter.push(first);
    interpreter.push(second);

    Ok(())
}

pub fn copy(interpreter: &mut Interpreter) -> crate::Result<()> {
    let mut original_stack = Vec::new();
    let mut copy_stack = Vec::new();

    match interpreter.pop()? {
        Object::Integer(n) => {
            for _ in 0..n {
                let obj = interpreter.pop()?;
                copy_stack.push(obj.clone());
                original_stack.push(obj);
            }

            while let Some(obj) = original_stack.pop() {
                interpreter.push(obj);
            }

            while let Some(obj) = copy_stack.pop() {
                interpreter.push(obj);
            }

            Ok(())
        },
        Object::Array(dest_idx) => {
            let obj = interpreter.pop()?;

            let source = match obj {
                Object::Array(idx) | Object::PackedArray(idx) => {
                    let arr = interpreter.arrays.get(idx)?;

                    Ok(arr)
                },
                _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
            }?;

            if source.access().is_exec_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let source = source.value().clone();

            let destination = interpreter.arrays.get_mut(dest_idx)?;

            if destination.access().is_read_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            for (index, obj) in source.into_iter().enumerate() {
                match destination.value_mut().get_mut(index) {
                    Some(dest_obj) => *dest_obj = obj,
                    None => return Err(Error::from(ErrorKind::RangeCheck)),
                }
            }

            interpreter.push(Object::Array(dest_idx));

            Ok(())
        },
        Object::Dictionary(idx) => {
            let source = interpreter.pop_dict()?.value().clone();
            let destination = interpreter.dicts.get_mut(idx)?;

            if destination.capacity() < source.len() {
                return Err(Error::from(ErrorKind::RangeCheck));
            }

            for (key, value) in source {
                destination.insert(key, value)?;
            }

            interpreter.push(Object::Dictionary(idx));

            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
    }
}

pub fn roll(interpreter: &mut Interpreter) -> crate::Result<()> {
    let mut top_stack = Vec::new();
    let mut bottom_stack = Vec::new();

    let j = interpreter.pop_int()?;
    let n = interpreter.pop_int()?;

    let j = (n + j) % n;
    for i in 0..n {
        let obj = interpreter.pop()?;
        if i < j {
            bottom_stack.push(obj);
        } else {
            top_stack.push(obj);
        }
    }

    while let Some(obj) = bottom_stack.pop() {
        interpreter.push(obj);
    }

    while let Some(obj) = top_stack.pop() {
        interpreter.push(obj);
    }

    Ok(())
}

pub fn index(interpreter: &mut Interpreter) -> crate::Result<()> {
    let idx = interpreter.pop_usize()?;

    if interpreter.operand_stack.is_empty() {
        return Err(Error::from(ErrorKind::RangeCheck));
    }

    let Some(idx) = (interpreter.operand_stack.len() - 1).checked_sub(idx) else {
        return Err(Error::from(ErrorKind::RangeCheck));
    };

    match interpreter.operand_stack.get(idx) {
        Some(obj) => {
            interpreter.push(obj.clone());
            Ok(())
        },
        None => Err(Error::from(ErrorKind::RangeCheck)),
    }
}

pub fn mark(interpreter: &mut Interpreter) -> crate::Result<()> {
    interpreter.push(Object::Mark);

    Ok(())
}

pub fn clear(interpreter: &mut Interpreter) -> crate::Result<()> {
    interpreter.operand_stack.clear();

    Ok(())
}

pub fn count(interpreter: &mut Interpreter) -> crate::Result<()> {
    let i = usize_to_i32(interpreter.operand_stack.len())?;

    interpreter.push(Object::Integer(i));

    Ok(())
}

pub fn counttomark(interpreter: &mut Interpreter) -> crate::Result<()> {
    let Some((u, _)) = interpreter
        .operand_stack
        .iter()
        .rev()
        .enumerate()
        .find(|(_, obj)| matches!(obj, Object::Mark))
    else {
        return Err(Error::from(ErrorKind::UnmatchedMark));
    };

    let i = usize_to_i32(u)?;

    interpreter.push(Object::Integer(i));

    Ok(())
}

pub fn cleartomark(interpreter: &mut Interpreter) -> crate::Result<()> {
    loop {
        match interpreter.pop() {
            Err(_) => return Err(Error::from(ErrorKind::UnmatchedMark)),
            Ok(obj) => {
                if matches!(obj, Object::Mark) {
                    break;
                }
            },
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    #[test]
    fn test_dup() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 dup".chars().into())?;

        assert_eq!(2, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_exch() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 exch".chars().into())?;

        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_exch_underflow() {
        let cases = ["1 exch", "exch"];

        for input in cases {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_pop() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 pop".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_pop_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("pop".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_simple() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 3 2 copy".chars().into())?;

        // Stack should be: 3 2 3 2 1 |
        assert_eq!(5, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(3)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(3)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_copy_simple_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("copy".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_composite() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 ] [ 4 5 6 ] copy".chars().into())?;

        // Stack should be: [ 1 2 3 ] |
        assert_eq!(1, interpreter.operand_stack.len());
        let Some(Object::Array(arr_idx)) = interpreter.operand_stack.pop() else {
            return Err("expected array object".into());
        };

        let arr = interpreter.arrays.get(arr_idx)?;

        assert_eq!(Some(Object::Integer(1)), arr.value().get(0).cloned());
        assert_eq!(Some(Object::Integer(2)), arr.value().get(1).cloned());
        assert_eq!(Some(Object::Integer(3)), arr.value().get(2).cloned());

        Ok(())
    }

    #[test]
    fn test_copy_composite_rangecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("[ 1 2 3 ] [ 4 5 ] copy".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_composite_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 [ 1 2 3 ] copy".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_composite_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("[ 1 2 3 ] copy".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_dictionary() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("<</key 1>> 1 dict copy".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());

        let dict = interpreter.pop_dict()?;
        assert_eq!(Object::Integer(1), dict.get("key".to_string()).cloned()?);

        Ok(())
    }

    #[test]
    fn test_roll() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 3 3 1 roll".chars().into())?;

        assert_eq!(3, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(3)), interpreter.operand_stack.pop());

        interpreter.operand_stack.clear();
        interpreter.evaluate("1 2 3 3 -1 roll".chars().into())?;

        assert_eq!(3, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(3)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_roll_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("roll".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_index() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 2 3 0 index", 3),
            ("1 2 3 1 index", 2),
            ("1 2 3 2 index", 1),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_index_rangecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 2 3 4 index".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_index_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("index".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_mark() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("mark".chars().into())?;

        assert!(matches!(
            interpreter.operand_stack.pop(),
            Some(Object::Mark)
        ));

        Ok(())
    }

    #[test]
    fn test_clear() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 3 clear".chars().into())?;

        assert_eq!(0, interpreter.operand_stack.len());

        Ok(())
    }

    #[test]
    fn test_count() -> Result<(), Box<dyn error::Error>> {
        let cases = [("count", 0), ("1 2 3 count", 3)];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_counttomark() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 mark 2 3 counttomark".chars().into())?;

        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_counttomark_unmatchedmark() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 2 3 counttomark 3".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::UnmatchedMark, result.unwrap_err().kind());
    }

    #[test]
    fn test_cleartomark() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 mark 2 3 cleartomark".chars().into())?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());

        Ok(())
    }
}
