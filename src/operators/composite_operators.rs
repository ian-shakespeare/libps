use std::{cell, rc};

use crate::{execution::ExecutionState, object::Object, stack::Stack, Error, ErrorKind};

fn i32_to_usize(i: i32) -> crate::Result<usize> {
    let u: usize = match i.try_into() {
        Ok(u) => u,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Unregistered,
                "failed to convert integer to usize",
            ))
        }
    };

    Ok(u)
}

fn usize_to_i32(u: usize) -> crate::Result<i32> {
    let i: i32 = match u.try_into() {
        Ok(i) => i,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Unregistered,
                "failed to usize into integer",
            ))
        }
    };

    Ok(i)
}

/// Creates an array of length int, each of whose elements is initialized with a null ob-
/// ject, and pushes this array on the operand stack. The int operand must be a non-
/// negative integer not greater than the maximum allowable array length (see
/// Appendix B). The array is allocated in local or global VM according to the current
/// VM allocation mode (see Section 3.7.2, “Local and Global VM” ).
///
/// Errors: LimitCheck, RangeCheck, StackUnderflow, TypeCheck, VmError
pub fn array(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(length)) => {
            if length < 0 {
                return Err(Error::new(
                    ErrorKind::RangeCheck,
                    "array cannot have negative length",
                ));
            }
            let length = i32_to_usize(length)?;
            let arr = vec![Object::Null; length];
            execution_state
                .operand_stack
                .push(Object::Array(rc::Rc::new(cell::RefCell::new(arr))));
            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Pushes a mark object on the operand stack (the same as the mark and << opera-
/// tors). The customary use of the [ operator is to mark the beginning of an indefi-
/// nitely long sequence of objects that will eventually be formed into a new array
/// object by a matching ] operator. See the discussion of array syntax in Section 3.2,
/// “Syntax,” and of array construction in Section 3.6, “Overview of Basic Opera-
/// tors.”
///
/// Errors: StackOverflow
pub fn startarray(execution_state: &mut ExecutionState) -> crate::Result<()> {
    execution_state.operand_stack.push(Object::Mark);
    Ok(())
}

/// Creates a new array of n elements (where n is the number of elements above the
/// topmost mark on the operand stack), stores those elements into the array, and re-
/// turns the array on the operand stack. The ] operator stores the topmost object
/// from the stack into element n − 1 of array and the bottommost one (the one im-
/// mediately above the mark) into element 0 of array. It removes all the array ele-
/// ments from the stack, as well as the mark object.
///
/// The array is allocated in local or global VM according to the current VM alloca-
/// tion mode. An invalidaccess error occurs if the array is in global VM and any of
/// the objects obj0 … obj n−1 are in local VM. See Section 3.7.2, “Local and Global
/// VM.”
///
/// Errors: StackOverflow, UnmatchedMark, VmError
pub fn endarray(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let mut temp_stack = Stack::new();

    while let Some(obj) = execution_state.operand_stack.pop() {
        match obj {
            Object::Mark => {
                let mut arr = Vec::with_capacity(temp_stack.count());
                while let Some(obj) = temp_stack.pop() {
                    arr.push(obj);
                }

                execution_state
                    .operand_stack
                    .push(Object::Array(rc::Rc::new(cell::RefCell::new(arr))));

                return Ok(());
            }
            _ => temp_stack.push(obj),
        }
    }

    Err(Error::from(ErrorKind::UnmatchedMark))
}

/// Returns the number of elements in the value of its operand if the operand is an
/// array, a packed array, or a string. If the operand is a dictionary, length returns the
/// current number of entries it contains (as opposed to its maximum capacity, which
/// is returned by maxlength). If the operand is a name object, the length returned is
/// the number of characters in the text string that defines it.
///
/// Errors: InvalidAccess, StackUnderflow, TypeCheck
pub fn length(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Array(arr)) => {
            let l = arr.borrow().len();
            let l = usize_to_i32(l)?;
            execution_state.operand_stack.push(Object::Integer(l));
            Ok(())
        }
        Some(_) => Err(Error::from(ErrorKind::TypeCheck)),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Returns a single element from the value of the first operand. If the first operand is
/// an array, a packed array, or a string, get treats the second operand as an index and
/// returns the element identified by the index, counting from 0. index must be in the
/// range 0 to n − 1, where n is the length of the array, packed array, or string. If it is
/// outside this range, a rangecheck error occurs.
///
/// If the first operand is a dictionary, get looks up the second operand as a key in the
/// dictionary and returns the associated value. If the key is not present in the diction-
/// ary, an undefined error occurs.
///
/// Errors: InvalidAccess, RangeCheck, StackUnderflow, TypeCheck, Undefined
pub fn get(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let index = execution_state.operand_stack.pop();
    let arr = execution_state.operand_stack.pop();

    match (arr, index) {
        (Some(arr), Some(index)) => match (arr, index) {
            (Object::Array(arr), Object::Integer(index)) => {
                let index = i32_to_usize(index)?;
                match arr.borrow().get(index) {
                    None => Err(Error::from(ErrorKind::RangeCheck)),
                    Some(value) => {
                        execution_state.operand_stack.push(value.clone());
                        Ok(())
                    }
                }
            }
            _ => Err(Error::from(ErrorKind::TypeCheck)),
        },
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Replaces a single element of the value of the first operand. If the first operand is an
/// array or a string, put treats the second operand as an index and stores the third
/// operand at the position identified by the index, counting from 0. index must be in
/// the range 0 to n − 1, where n is the length of the array or string. If it is outside this
/// range, a rangecheck error occurs.
///
/// If the first operand is a dictionary, put uses the second operand as a key and the
/// third operand as a value, and stores this key-value pair into dict. If key is already
/// present as a key in dict, put simply replaces its value by any; otherwise, put creates
/// a new entry for key and associates any with it. In LanguageLevel 1, if dict is already
/// full, a dictfull error occurs.
///
/// If the value of array or dict is in global VM and any is a composite object whose
/// value is in local VM, an invalidaccess error occurs (see Section 3.7.2, “Local and
/// Global VM”).
///
/// Errors: DictFull, InvalidAccess, RangeCheck, StackUnderflow, TypeCheck
pub fn put(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let value = execution_state.operand_stack.pop();
    let index = execution_state.operand_stack.pop();
    let arr = execution_state.operand_stack.pop();

    match (arr, index, value) {
        (Some(arr), Some(index), Some(value)) => match (arr, index) {
            (Object::Array(arr), Object::Integer(index)) => {
                let index = i32_to_usize(index)?;
                match arr.borrow_mut().get_mut(index) {
                    None => Err(Error::from(ErrorKind::RangeCheck)),
                    Some(obj) => {
                        *obj = value;
                        Ok(())
                    }
                }
            }
            _ => Err(Error::from(ErrorKind::TypeCheck)),
        },
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Creates a new array, packed array, or string object whose value consists of some
/// subsequence of the original array, packed array, or string. The subsequence con-
/// sists of count elements starting at the specified index in the original object. The ele-
/// ments in the subsequence are shared between the original and new objects (see
/// Section 3.3.1, “Simple and Composite Objects”).
///
/// The returned subarray or substring is an ordinary array, packed array, or string
/// object whose length is count and whose elements are indexed starting at 0. The
/// element at index 0 in the result is the same as the element at index in the original
/// object.
///
/// getinterval requires index to be a valid index in the original object and count to be
/// a nonnegative integer such that index + count is not greater than the length of the
/// original object.
///
/// Errors: InvalidAccess, RangeCheck, StackUnderflow, TypeCheck
pub fn getinterval(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let mut temp_stack = Stack::new();

    let count = execution_state.operand_stack.pop();
    let index = execution_state.operand_stack.pop();
    let arr = execution_state.operand_stack.pop();

    match (arr, index, count) {
        (Some(arr), Some(index), Some(count)) => match (arr, index, count) {
            (Object::Array(arr), Object::Integer(index), Object::Integer(count)) => {
                let index: usize = match index.try_into() {
                    Ok(index) => index,
                    Err(_) => return Err(Error::from(ErrorKind::RangeCheck)),
                };

                let count: usize = match count.try_into() {
                    Ok(count) => count,
                    Err(_) => return Err(Error::from(ErrorKind::RangeCheck)),
                };

                let arr = arr.borrow();
                if index >= arr.len() {
                    return Err(Error::from(ErrorKind::RangeCheck));
                }

                for i in index..(index + count) {
                    match arr.get(i) {
                        None => return Err(Error::from(ErrorKind::RangeCheck)),
                        Some(obj) => temp_stack.push(obj.clone()),
                    }
                }

                let subarray: Vec<Object> = temp_stack.into();
                execution_state
                    .operand_stack
                    .push(Object::Array(rc::Rc::new(cell::RefCell::new(subarray))));

                Ok(())
            }
            _ => Err(Error::from(ErrorKind::TypeCheck)),
        },
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Replaces a subsequence of the elements of the first operand by the entire contents
/// of the third operand. The subsequence that is replaced begins at index in the first
/// operand; its length is the same as the length of the third operand.
///
/// The objects are copied from the third operand to the first, as if by a sequence of
/// individual get and put operations. In the case of arrays, if the copied elements are
/// themselves composite objects, the values of those objects are shared between
/// array2 and array1 (see Section 3.3.1, “Simple and Composite Objects”).
///
/// putinterval requires index to be a valid index in array1 or string1 such that index
/// plus the length of array2 or string2 is not greater than the length of array1 or string1.
///
/// If the value of array1 is in global VM and any of the elements copied from array2 or
/// packedarray2 are composite objects whose values are in local VM, an invalidaccess
/// error occurs (see Section 3.7.2, “Local and Global VM”).
///
/// Errors: InvalidAccess, RangeCheck, StackUnderflow, TypeCheck
pub fn putinterval(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let source = execution_state.operand_stack.pop();
    let index = execution_state.operand_stack.pop();
    let destination = execution_state.operand_stack.pop();

    match (destination, index, source) {
        (Some(destination), Some(index), Some(source)) => match (destination, index, source) {
            (Object::Array(destination), Object::Integer(index), Object::Array(source)) => {
                let index: usize = match index.try_into() {
                    Ok(index) => index,
                    Err(_) => return Err(Error::from(ErrorKind::RangeCheck)),
                };
                let mut destination = destination.borrow_mut();
                let source = source.borrow();
                for (offset, obj) in source.iter().enumerate() {
                    match destination.get_mut(index + offset) {
                        Some(dest_obj) => *dest_obj = obj.clone(),
                        None => return Err(Error::from(ErrorKind::RangeCheck)),
                    }
                }
                Ok(())
            }
            _ => Err(Error::from(ErrorKind::TypeCheck)),
        },
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Stores the objects any0 to any n−1 from the operand stack into array, where n is the
/// length of array. The astore operator first removes the array operand from the stack
/// and determines its length. It then removes that number of objects from the stack,
/// storing the topmost one into element n − 1 of array and the bottommost one into
/// element 0. Finally, it pushes array back on the stack. Note that an astore operation
/// cannot be performed on packed arrays.
///
/// If the value of array is in global VM and any of the objects any0 through any n−1 are
/// composite objects whose values are in local VM, an invalidaccess error occurs (see
/// Section 3.7.2, “Local and Global VM”).
///
/// Errors: InvalidAccess, StackUnderflow, TypeCheck
pub fn astore(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Array(arr)) => {
            for dest_obj in arr.borrow_mut().iter_mut().rev() {
                match execution_state.operand_stack.pop() {
                    None => return Err(Error::from(ErrorKind::StackUnderflow)),
                    Some(obj) => *dest_obj = obj,
                }
            }
            execution_state
                .operand_stack
                .push(Object::Array(arr.clone()));

            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Successively pushes all n elements of array or packedarray on the operand stack
/// (where n is the length of the operand), and then pushes the operand itself.
///
/// Errors: InvalidAccess, StackOverflow, StackUnderflow, TypeCheck
pub fn aload(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Array(arr)) => {
            for obj in arr.borrow().iter() {
                execution_state.operand_stack.push(obj.clone());
            }

            execution_state.operand_stack.push(Object::Array(arr));
            Ok(())
        }
        Some(_) => Err(Error::from(ErrorKind::TypeCheck)),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    #[test]
    fn test_array() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        for num in 0..5 {
            execution_state.operand_stack.push(Object::Integer(num));

            assert!(array(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());

            let Some(Object::Array(arr)) = execution_state.operand_stack.pop() else {
                return Err("expected array object".into());
            };

            assert_eq!(num, arr.borrow().len().try_into()?);

            for obj in arr.borrow().iter() {
                assert_eq!(Object::Null, obj.clone());
            }
        }

        assert!(array(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

        execution_state.operand_stack.push(Object::Integer(-1));
        assert!(array(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::RangeCheck));

        Ok(())
    }

    #[test]
    fn test_startarray() {
        let mut execution_state = ExecutionState::new();

        assert!(startarray(&mut execution_state).is_ok());
        assert_eq!(1, execution_state.operand_stack.count());
        assert!(
            if let Some(Object::Mark) = execution_state.operand_stack.pop() {
                true
            } else {
                false
            },
            "expected mark object"
        );
    }

    #[test]
    fn test_endarray() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        for length in 0..5 {
            execution_state.operand_stack.push(Object::Mark);
            for _ in 0..length {
                execution_state.operand_stack.push(Object::Integer(0));
            }

            assert!(endarray(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());

            let Some(Object::Array(arr)) = execution_state.operand_stack.pop() else {
                return Err("expected array object".into());
            };

            assert_eq!(length, arr.borrow().len());

            for i in 0..length {
                assert_eq!(Object::Integer(0), arr.borrow()[i]);
            }
        }

        assert!(endarray(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::UnmatchedMark));

        Ok(())
    }

    #[test]
    fn test_length() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        for arr_length in 0..5 {
            let arr = vec![Object::Null; arr_length];
            execution_state
                .operand_stack
                .push(Object::Array(rc::Rc::new(cell::RefCell::new(arr))));

            assert!(length(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());

            let Some(Object::Integer(value)) = execution_state.operand_stack.pop() else {
                return Err("expected integer object".into());
            };

            assert_eq!(arr_length, value.try_into()?);
        }

        Ok(())
    }

    #[test]
    fn test_get() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let arr: Vec<Object> = (0..5).map(|n| Object::Integer(n)).collect();
        let arr = rc::Rc::new(cell::RefCell::new(arr));

        for i in 0..5 {
            execution_state
                .operand_stack
                .push(Object::Array(arr.clone()));
            execution_state.operand_stack.push(Object::Integer(i));

            assert!(get(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());

            let Some(Object::Integer(n)) = execution_state.operand_stack.pop() else {
                return Err("expected integer object".into());
            };

            assert_eq!(i, n);
        }

        execution_state
            .operand_stack
            .push(Object::Array(arr.clone()));
        execution_state.operand_stack.push(Object::Integer(1000));
        assert!(get(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::RangeCheck));

        execution_state.operand_stack.push(Object::Integer(0));
        execution_state.operand_stack.push(Object::Integer(0));
        assert!(get(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::TypeCheck));

        assert!(get(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

        Ok(())
    }

    #[test]
    fn test_put() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let arr: Vec<Object> = vec![Object::Null; 5];
        let arr = rc::Rc::new(cell::RefCell::new(arr));

        for (i, s) in ["test1", "test2", "test3", "test4", "test5"]
            .iter()
            .enumerate()
        {
            execution_state
                .operand_stack
                .push(Object::Array(arr.clone()));
            execution_state
                .operand_stack
                .push(Object::Integer(i.try_into()?));
            execution_state
                .operand_stack
                .push(Object::String(s.to_string()));

            assert!(put(&mut execution_state).is_ok());
            assert_eq!(0, execution_state.operand_stack.count());
            assert_eq!(Object::String(s.to_string()), arr.borrow()[i]);
        }

        execution_state
            .operand_stack
            .push(Object::Array(arr.clone()));
        execution_state.operand_stack.push(Object::Integer(1000));
        execution_state.operand_stack.push(Object::Integer(0));
        assert!(put(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::RangeCheck));

        execution_state.operand_stack.push(Object::Integer(0));
        execution_state.operand_stack.push(Object::Integer(0));
        execution_state.operand_stack.push(Object::Integer(0));
        assert!(put(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::TypeCheck));

        assert!(put(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

        Ok(())
    }

    #[test]
    fn test_getinterval() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let arr = vec![
            Object::Integer(0),
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
        ];
        let arr = rc::Rc::new(cell::RefCell::new(arr));

        execution_state
            .operand_stack
            .push(Object::Array(arr.clone()));
        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Integer(3));

        assert!(getinterval(&mut execution_state).is_ok());
        assert_eq!(1, execution_state.operand_stack.count());

        let Some(Object::Array(subarray)) = execution_state.operand_stack.pop() else {
            return Err("expected array object".into());
        };
        let subarray = subarray.borrow();

        assert_eq!(3, subarray.len());
        assert_eq!(Object::Integer(1), subarray[0]);
        assert_eq!(Object::Integer(2), subarray[1]);
        assert_eq!(Object::Integer(3), subarray[2]);

        let range_check_cases = [
            (Object::Integer(-1), Object::Integer(0)),
            (Object::Integer(0), Object::Integer(-1)),
            (Object::Integer(1000), Object::Integer(0)),
            (Object::Integer(0), Object::Integer(1000)),
        ];

        for (index, count) in range_check_cases {
            execution_state
                .operand_stack
                .push(Object::Array(arr.clone()));
            execution_state.operand_stack.push(index);
            execution_state.operand_stack.push(count);
            assert!(
                getinterval(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::RangeCheck)
            );
        }

        assert!(
            getinterval(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::StackUnderflow)
        );

        Ok(())
    }

    #[test]
    fn test_putinterval() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let arr = rc::Rc::new(cell::RefCell::new(vec![
            Object::Integer(0),
            Object::Integer(1),
            Object::Integer(2),
            Object::Integer(3),
            Object::Integer(4),
        ]));

        let subarray = rc::Rc::new(cell::RefCell::new(vec![
            Object::Integer(5),
            Object::Integer(6),
            Object::Integer(7),
        ]));

        execution_state
            .operand_stack
            .push(Object::Array(arr.clone()));
        execution_state.operand_stack.push(Object::Integer(1));
        execution_state
            .operand_stack
            .push(Object::Array(subarray.clone()));

        assert!(putinterval(&mut execution_state).is_ok());
        assert_eq!(0, execution_state.operand_stack.count());

        let borrowed_arr = arr.borrow();
        assert_eq!(Object::Integer(0), borrowed_arr[0]);
        assert_eq!(Object::Integer(5), borrowed_arr[1]);
        assert_eq!(Object::Integer(6), borrowed_arr[2]);
        assert_eq!(Object::Integer(7), borrowed_arr[3]);
        assert_eq!(Object::Integer(4), borrowed_arr[4]);
        drop(borrowed_arr);

        let range_check_cases = [
            Object::Integer(1000),
            Object::Integer(-1),
            Object::Integer(3),
        ];

        for case in range_check_cases {
            execution_state
                .operand_stack
                .push(Object::Array(arr.clone()));
            execution_state.operand_stack.push(case);
            execution_state
                .operand_stack
                .push(Object::Array(subarray.clone()));
            assert!(
                putinterval(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::RangeCheck)
            );
        }

        assert!(
            putinterval(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::StackUnderflow)
        );

        Ok(())
    }

    #[test]
    fn test_astore() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let arr = rc::Rc::new(cell::RefCell::new(vec![Object::Null; 3]));

        execution_state.operand_stack.push(Object::Integer(0));
        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Integer(2));
        execution_state
            .operand_stack
            .push(Object::Array(arr.clone()));

        assert!(astore(&mut execution_state).is_ok());
        assert_eq!(1, execution_state.operand_stack.count());

        let borrowed_arr = arr.borrow();
        assert_eq!(Object::Integer(0), borrowed_arr[0]);
        assert_eq!(Object::Integer(1), borrowed_arr[1]);
        assert_eq!(Object::Integer(2), borrowed_arr[2]);
        drop(borrowed_arr);

        // TODO: test for errors

        Ok(())
    }

    #[test]
    fn test_aload() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let arr = rc::Rc::new(cell::RefCell::new(vec![
            Object::Integer(0),
            Object::Integer(1),
            Object::Integer(2),
        ]));

        execution_state
            .operand_stack
            .push(Object::Array(arr.clone()));

        assert!(aload(&mut execution_state).is_ok());
        assert_eq!(4, execution_state.operand_stack.count());

        execution_state.operand_stack.pop();
        let borrowed_arr = arr.borrow();
        assert_eq!(Object::Integer(0), borrowed_arr[0]);
        assert_eq!(Object::Integer(1), borrowed_arr[1]);
        assert_eq!(Object::Integer(2), borrowed_arr[2]);
        drop(borrowed_arr);

        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(0)),
            execution_state.operand_stack.pop()
        );

        // TODO: test for errors

        Ok(())
    }
}
