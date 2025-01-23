use crate::{execution::ExecutionState, object::Object, stack::Stack, Error, ErrorKind};

/// Duplicates the top element on the operand stack. dup copies only the object; the
/// value of a composite object is not copied but is shared. See Section 3.3, “Data
/// Types and Objects.”
///
/// Errors: StackOverflow, StackUnderflow
pub fn dup(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.top() {
        None => Err(Error::from(ErrorKind::StackUnderflow)),
        Some(top) => {
            execution_state.operand_stack.push(top.clone());
            Ok(())
        }
    }
}

/// Exchanges the top two elements on the operand stack.
///
/// Errors: StackUnderflow
pub fn exch(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let first = execution_state.operand_stack.pop();
    let second = execution_state.operand_stack.pop();

    match (first, second) {
        (Some(first), Some(second)) => {
            execution_state.operand_stack.push(first);
            execution_state.operand_stack.push(second);
            Ok(())
        }
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Removes the top element from the operand stack and discards it.
///
/// Errors: StackUnderflow
pub fn pop(execution_state: &mut ExecutionState) -> crate::Result<()> {
    if execution_state.operand_stack.pop().is_none() {
        Err(Error::from(ErrorKind::StackUnderflow))
    } else {
        Ok(())
    }
}

/// Performs two entirely different functions, depending on the type of the topmost
/// operand.
///
/// In the first form, where the top element on the operand stack is a nonnegative in-
/// teger n, copy pops n from the stack and duplicates the top n elements on the stack
/// as shown above. This form of copy operates only on the objects themselves, not
/// on the values of composite objects.
///
/// Errors: InvalidAccess, RangeCheck, StackOverflow, StackUnderflow, TypeCheck
pub fn copy(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let mut original_stack: Stack<Object> = Stack::new();
    let mut copy_stack: Stack<Object> = Stack::new();

    match execution_state.operand_stack.pop() {
        Some(Object::Integer(n)) => {
            for _ in 0..n {
                match execution_state.operand_stack.pop() {
                    None => return Err(Error::from(ErrorKind::StackUnderflow)),
                    Some(obj) => {
                        copy_stack.push(obj.clone());
                        original_stack.push(obj);
                    }
                }
            }

            while let Some(obj) = original_stack.pop() {
                execution_state.operand_stack.push(obj);
            }

            while let Some(obj) = copy_stack.pop() {
                execution_state.operand_stack.push(obj);
            }

            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Performs a circular shift of the objects any n−1 through any0 on the operand stack
/// by the amount j. Positive j indicates upward motion on the stack, whereas nega-
/// tive j indicates downward motion.
///
/// n must be a nonnegative integer and j must be an integer. roll first removes these
/// operands from the stack; there must be at least n additional elements. It then per-
/// forms a circular shift of these n elements by j positions.
///
/// If j is positive, each shift consists of removing an element from the top of the stack
/// and inserting it between element n − 1 and element n of the stack, moving all in-
/// tervening elements one level higher on the stack. If j is negative, each shift consists
/// of removing element n − 1 of the stack and pushing it on the top of the stack,
/// moving all intervening elements one level lower on the stack.
///
/// Errors: RangeCheck, StackUnderflow, TypeCheck
pub fn roll(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let mut temp_stack: Stack<Object> = Stack::new();
    let mut shift_stack: Stack<Object> = Stack::new();
    let j = execution_state.operand_stack.pop();
    let n = execution_state.operand_stack.pop();

    match (n, j) {
        (Some(Object::Integer(n)), Some(Object::Integer(j))) => {
            let j = j % n;
            for i in 0..n {
                match execution_state.operand_stack.pop() {
                    None => return Err(Error::from(ErrorKind::StackUnderflow)),
                    Some(obj) => {
                        if i < j {
                            shift_stack.push(obj);
                        } else {
                            temp_stack.push(obj);
                        }
                    }
                }
            }

            while let Some(obj) = shift_stack.pop() {
                execution_state.operand_stack.push(obj);
            }

            while let Some(obj) = temp_stack.pop() {
                execution_state.operand_stack.push(obj);
            }

            Ok(())
        }
        (Some(_), Some(_)) => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Removes the nonnegative integer n from the operand stack, counts down to the
/// nth element from the top of the stack, and pushes a copy of that element on the
/// stack.
///
/// Errors: RangeCheck, StackUnderflow, TypeCheck
pub fn index(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let mut temp_stack: Stack<Object> = Stack::new();

    match execution_state.operand_stack.pop() {
        Some(Object::Integer(n)) => {
            for _ in 0..n {
                match execution_state.operand_stack.pop() {
                    None => return Err(Error::from(ErrorKind::StackUnderflow)),
                    Some(obj) => temp_stack.push(obj),
                }
            }

            let Some(target) = execution_state.operand_stack.top().cloned() else {
                return Err(Error::from(ErrorKind::StackUnderflow));
            };

            while let Some(obj) = temp_stack.pop() {
                execution_state.operand_stack.push(obj);
            }

            execution_state.operand_stack.push(target);

            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Pushes a mark object on the operand stack. All marks are identical, and the oper-
/// and stack may contain any number of them at once.
///
/// The primary use of marks is to indicate the stack position of the beginning of an
/// indefinitely long list of operands being passed to an operator or procedure. The ]
/// operator (array construction) is the most common operator that works this way;
/// it treats as operands all elements of the stack down to a mark that was pushed by
/// the [ operator ([ is a synonym for mark). It is possible to define procedures that
/// work similarly. Operators such as counttomark and cleartomark are useful within
/// such procedures.
///
/// Errors: StackOverflow
pub fn mark(execution_state: &mut ExecutionState) -> crate::Result<()> {
    execution_state.operand_stack.push(Object::Mark);

    Ok(())
}

/// Pops all objects from the operand stack and discards them.
///
/// Errors: None
pub fn clear(execution_state: &mut ExecutionState) -> crate::Result<()> {
    execution_state.operand_stack.clear();

    Ok(())
}

/// Counts the number of items on the operand stack and pushes this count on the
/// operand stack.
///
/// Errors: StackOverflow
pub fn count(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let stack_count = match execution_state.operand_stack.count().try_into() {
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Unregistered,
                "failed to convert count usize to int32",
            ))
        }
        Ok(stack_count) => stack_count,
    };
    execution_state
        .operand_stack
        .push(Object::Integer(stack_count));

    Ok(())
}

/// Counts the number of objects on the operand stack, starting with the top element
/// and continuing down to but not including the first mark encountered. obj1
/// through obj n are any objects other than marks.
///
/// Errors: StackOverflow, UnmatchedMark
pub fn counttomark(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let mut temp_stack: Stack<Object> = Stack::new();

    while let Some(obj) = execution_state.operand_stack.pop() {
        if let Object::Mark = obj {
            execution_state.operand_stack.push(Object::Mark);
            break;
        }

        temp_stack.push(obj);
    }

    let stack_count = match temp_stack.count().try_into() {
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Unregistered,
                "failed to convert count usize to int32",
            ))
        }
        Ok(stack_count) => stack_count,
    };

    while let Some(obj) = temp_stack.pop() {
        execution_state.operand_stack.push(obj);
    }

    execution_state
        .operand_stack
        .push(Object::Integer(stack_count));

    Ok(())
}

/// Pops entries from the operand stack repeatedly until it encounters a mark, which
/// it also pops from the stack. obj1 through obj n are any objects other than marks.
///
/// Errors: UnmatchedMark
pub fn cleartomark(execution_state: &mut ExecutionState) -> crate::Result<()> {
    while let Some(obj) = execution_state.operand_stack.pop() {
        if let Object::Mark = obj {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dup() {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(1));

        assert!(dup(&mut execution_state).is_ok());
        assert_eq!(2, execution_state.operand_stack.count());
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop()
        );
        assert!(dup(&mut execution_state).is_err());
    }

    #[test]
    fn test_exch() {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Integer(2));

        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.top().cloned()
        );
        assert!(exch(&mut execution_state).is_ok());
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.pop()
        );
        assert!(exch(&mut execution_state).is_err());

        execution_state.operand_stack.push(Object::Integer(1));
        assert!(exch(&mut execution_state).is_err());
    }

    #[test]
    fn test_pop() {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(1));

        assert!(pop(&mut execution_state).is_ok());
        assert!(execution_state.operand_stack.top().is_none());
        assert!(pop(&mut execution_state).is_err());
    }

    #[test]
    fn test_copy() {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(3));
        execution_state.operand_stack.push(Object::Integer(2));

        assert!(copy(&mut execution_state).is_ok());

        // Stack should be: 3 2 3 2 1 |
        assert_eq!(5, execution_state.operand_stack.count());
        assert_eq!(
            Some(Object::Integer(3)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(3)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop()
        );

        assert!(copy(&mut execution_state).is_err());
    }

    #[test]
    fn test_roll() {
        let mut execution_state = ExecutionState::new();

        // 3 2 1 |
        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(3));
        // n = 3, j = 2
        execution_state.operand_stack.push(Object::Integer(3));
        execution_state.operand_stack.push(Object::Integer(2));

        assert!(roll(&mut execution_state).is_ok());

        // 1 3 2 |
        assert_eq!(3, execution_state.operand_stack.count());
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(3)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.pop()
        );

        assert!(roll(&mut execution_state).is_err());
    }

    #[test]
    fn test_index() {
        let mut execution_state = ExecutionState::new();

        assert!(index(&mut execution_state).is_err());

        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(3));
        execution_state.operand_stack.push(Object::Integer(2));

        assert!(index(&mut execution_state).is_ok());
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop()
        );

        execution_state.operand_stack.push(Object::Integer(1));
        assert!(index(&mut execution_state).is_ok());
        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.pop()
        );

        execution_state.operand_stack.push(Object::Integer(0));
        assert!(index(&mut execution_state).is_ok());
        assert_eq!(
            Some(Object::Integer(3)),
            execution_state.operand_stack.pop()
        );
    }

    #[test]
    fn test_mark() {
        let mut execution_state = ExecutionState::new();

        assert!(mark(&mut execution_state).is_ok());
        assert!(match execution_state.operand_stack.top() {
            Some(Object::Mark) => true,
            _ => false,
        });
    }

    #[test]
    fn test_clear() {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(3));

        assert_eq!(3, execution_state.operand_stack.count());
        assert!(clear(&mut execution_state).is_ok());
        assert_eq!(0, execution_state.operand_stack.count());
    }

    #[test]
    fn test_count() {
        let mut execution_state = ExecutionState::new();

        assert!(count(&mut execution_state).is_ok());
        assert_eq!(
            Some(Object::Integer(0)),
            execution_state.operand_stack.top().cloned()
        );

        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(3));

        assert!(count(&mut execution_state).is_ok());
        assert_eq!(
            Some(Object::Integer(4)),
            execution_state.operand_stack.top().cloned()
        );
        assert_eq!(5, execution_state.operand_stack.count());
    }

    #[test]
    fn test_counttomark() {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Mark);
        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(3));

        assert!(counttomark(&mut execution_state).is_ok());
        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.top().cloned()
        );

        assert!(counttomark(&mut execution_state).is_ok());

        // Stack should be: 3 2 3 2 MARK 1 |
        assert_eq!(
            Some(Object::Integer(3)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(3)),
            execution_state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            execution_state.operand_stack.pop()
        );
        assert!(match execution_state.operand_stack.pop() {
            Some(Object::Mark) => true,
            _ => false,
        });
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop()
        );
    }

    #[test]
    fn test_cleartomark() {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(1));
        execution_state.operand_stack.push(Object::Mark);
        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(3));

        assert!(cleartomark(&mut execution_state).is_ok());
        assert_eq!(1, execution_state.operand_stack.count());
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop()
        );
    }
}
