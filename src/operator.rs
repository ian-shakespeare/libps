use crate::{object::Object, stack::Stack, Error, ErrorKind};

fn is_valid_real(real: f64) -> bool {
    real.is_finite() && !real.is_nan()
}

pub fn dup(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.top() {
        None => Err(Error::from(ErrorKind::StackUnderflow)),
        Some(top) => {
            operand_stack.push(top.clone());
            Ok(())
        }
    }
}

pub fn exch(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let first = operand_stack.pop();
    let second = operand_stack.pop();

    match (first, second) {
        (Some(first), Some(second)) => {
            operand_stack.push(first);
            operand_stack.push(second);
            Ok(())
        }
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn pop(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    if operand_stack.pop().is_none() {
        Err(Error::from(ErrorKind::StackUnderflow))
    } else {
        Ok(())
    }
}

pub fn copy(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let mut original_stack: Stack<Object> = Stack::new();
    let mut copy_stack: Stack<Object> = Stack::new();

    match operand_stack.pop() {
        Some(Object::Integer(n)) => {
            for _ in 0..n {
                match operand_stack.pop() {
                    None => return Err(Error::from(ErrorKind::StackUnderflow)),
                    Some(obj) => {
                        copy_stack.push(obj.clone());
                        original_stack.push(obj);
                    }
                }
            }

            while let Some(obj) = original_stack.pop() {
                operand_stack.push(obj);
            }

            while let Some(obj) = copy_stack.pop() {
                operand_stack.push(obj);
            }

            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn roll(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let mut temp_stack: Stack<Object> = Stack::new();
    let mut shift_stack: Stack<Object> = Stack::new();
    let j = operand_stack.pop();
    let n = operand_stack.pop();

    match (n, j) {
        (Some(Object::Integer(n)), Some(Object::Integer(j))) => {
            let j = j % n;
            for i in 0..n {
                match operand_stack.pop() {
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
                operand_stack.push(obj);
            }

            while let Some(obj) = temp_stack.pop() {
                operand_stack.push(obj);
            }

            Ok(())
        }
        (Some(_), Some(_)) => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn index(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let mut temp_stack: Stack<Object> = Stack::new();

    match operand_stack.pop() {
        Some(Object::Integer(n)) => {
            for _ in 0..n {
                match operand_stack.pop() {
                    None => return Err(Error::from(ErrorKind::StackUnderflow)),
                    Some(obj) => temp_stack.push(obj),
                }
            }

            let Some(target) = operand_stack.top().cloned() else {
                return Err(Error::from(ErrorKind::StackUnderflow));
            };

            while let Some(obj) = temp_stack.pop() {
                operand_stack.push(obj);
            }

            operand_stack.push(target);

            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn mark(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    operand_stack.push(Object::Mark);

    Ok(())
}

pub fn clear(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    operand_stack.clear();

    Ok(())
}

pub fn count(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let stack_count = match operand_stack.count().try_into() {
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Unregistered,
                "failed to convert count usize to int32",
            ))
        }
        Ok(stack_count) => stack_count,
    };
    operand_stack.push(Object::Integer(stack_count));

    Ok(())
}

pub fn counttomark(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let mut temp_stack: Stack<Object> = Stack::new();

    while let Some(obj) = operand_stack.pop() {
        if let Object::Mark = obj {
            operand_stack.push(Object::Mark);
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
        operand_stack.push(obj);
    }

    operand_stack.push(Object::Integer(stack_count));

    Ok(())
}

pub fn cleartomark(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    while let Some(obj) = operand_stack.pop() {
        if let Object::Mark = obj {
            break;
        }
    }

    Ok(())
}

pub fn add(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let rhs = operand_stack.pop();
    let lhs = operand_stack.pop();

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => {
                match lhs.checked_add(rhs) {
                    Some(total) => operand_stack.push(Object::Integer(total)),
                    None => {
                        let total = f64::from(lhs) + f64::from(rhs);
                        if !is_valid_real(total) {
                            return Err(Error::from(ErrorKind::UndefinedResult));
                        }
                        operand_stack.push(Object::Real(total));
                    }
                }
                Ok(())
            }
            (Object::Integer(lhs), Object::Real(rhs)) => {
                let total = f64::from(lhs) + rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Integer(rhs)) => {
                let total = lhs + f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Real(rhs)) => {
                let total = lhs + rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected numerics")),
        },
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn div(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let rhs = operand_stack.pop();
    let lhs = operand_stack.pop();

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => {
                let total = f64::from(lhs) / f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Integer(lhs), Object::Real(rhs)) => {
                let total = f64::from(lhs) / rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Integer(rhs)) => {
                let total = lhs / f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Real(rhs)) => {
                let total = lhs / rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected numerics")),
        },
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn idiv(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let rhs = operand_stack.pop();
    let lhs = operand_stack.pop();

    match (lhs, rhs) {
        (Some(Object::Integer(lhs)), Some(Object::Integer(rhs))) => match lhs.checked_div(rhs) {
            None => Err(Error::new(ErrorKind::UndefinedResult, "division by zero")),
            Some(total) => {
                operand_stack.push(Object::Integer(total));
                Ok(())
            }
        },
        (Some(_), Some(_)) => Err(Error::new(ErrorKind::TypeCheck, "expected integers")),
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn imod(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let rhs = operand_stack.pop();
    let lhs = operand_stack.pop();

    match (lhs, rhs) {
        (Some(Object::Integer(lhs)), Some(Object::Integer(rhs))) => match lhs.checked_rem(rhs) {
            None => Err(Error::new(ErrorKind::UndefinedResult, "division by zero")),
            Some(total) => {
                operand_stack.push(Object::Integer(total));
                Ok(())
            }
        },
        (Some(_), Some(_)) => Err(Error::new(ErrorKind::TypeCheck, "expected integers")),
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn mul(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let rhs = operand_stack.pop();
    let lhs = operand_stack.pop();

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => {
                match lhs.checked_mul(rhs) {
                    Some(total) => operand_stack.push(Object::Integer(total)),
                    None => {
                        let total = f64::from(lhs) * f64::from(rhs);
                        if !is_valid_real(total) {
                            return Err(Error::from(ErrorKind::UndefinedResult));
                        }
                        operand_stack.push(Object::Real(total));
                    }
                }
                Ok(())
            }
            (Object::Integer(lhs), Object::Real(rhs)) => {
                let total = f64::from(lhs) * rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Integer(rhs)) => {
                let total = lhs * f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Real(rhs)) => {
                let total = lhs * rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected numerics")),
        },
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn sub(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let rhs = operand_stack.pop();
    let lhs = operand_stack.pop();

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => {
                match lhs.checked_sub(rhs) {
                    Some(total) => operand_stack.push(Object::Integer(total)),
                    None => {
                        let total = f64::from(lhs) - f64::from(rhs);
                        if !is_valid_real(total) {
                            return Err(Error::from(ErrorKind::UndefinedResult));
                        }
                        operand_stack.push(Object::Real(total));
                    }
                }
                Ok(())
            }
            (Object::Integer(lhs), Object::Real(rhs)) => {
                let total = f64::from(lhs) - rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Integer(rhs)) => {
                let total = lhs - f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Real(rhs)) => {
                let total = lhs - rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected numerics")),
        },
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

pub fn abs(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.pop() {
        Some(Object::Integer(value)) => {
            match value.checked_abs() {
                Some(total) => operand_stack.push(Object::Integer(total)),
                None => {
                    let total = f64::from(value).abs();
                    if !is_valid_real(total) {
                        return Err(Error::from(ErrorKind::UndefinedResult));
                    }
                    operand_stack.push(Object::Real(total));
                }
            }
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.abs();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected numeric")),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dup() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        operand_stack.push(Object::Integer(1));

        assert!(dup(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(2, operand_stack.count());
        assert_eq!(Some(Object::Integer(1)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(1)), operand_stack.pop());
        assert!(dup(&mut operand_stack, &mut execution_stack).is_err());
    }

    #[test]
    fn test_exch() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        operand_stack.push(Object::Integer(1));
        operand_stack.push(Object::Integer(2));

        assert_eq!(Some(Object::Integer(2)), operand_stack.top().cloned());
        assert!(exch(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(Some(Object::Integer(1)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), operand_stack.pop());
        assert!(exch(&mut operand_stack, &mut execution_stack).is_err());

        operand_stack.push(Object::Integer(1));
        assert!(exch(&mut operand_stack, &mut execution_stack).is_err());
    }

    #[test]
    fn test_pop() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        operand_stack.push(Object::Integer(1));

        assert!(pop(&mut operand_stack, &mut execution_stack).is_ok());
        assert!(operand_stack.top().is_none());
        assert!(pop(&mut operand_stack, &mut execution_stack).is_err());
    }

    #[test]
    fn test_copy() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        operand_stack.push(Object::Integer(1));
        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(3));
        operand_stack.push(Object::Integer(2));

        assert!(copy(&mut operand_stack, &mut execution_stack).is_ok());

        // Stack should be: 3 2 3 2 1 |
        assert_eq!(5, operand_stack.count());
        assert_eq!(Some(Object::Integer(3)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(3)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(1)), operand_stack.pop());

        assert!(copy(&mut operand_stack, &mut execution_stack).is_err());
    }

    #[test]
    fn test_roll() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        // 3 2 1 |
        operand_stack.push(Object::Integer(1));
        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(3));
        // n = 3, j = 2
        operand_stack.push(Object::Integer(3));
        operand_stack.push(Object::Integer(2));

        assert!(roll(&mut operand_stack, &mut execution_stack).is_ok());

        // 1 3 2 |
        assert_eq!(3, operand_stack.count());
        assert_eq!(Some(Object::Integer(1)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(3)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), operand_stack.pop());

        assert!(roll(&mut operand_stack, &mut execution_stack).is_err());
    }

    #[test]
    fn test_index() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        assert!(index(&mut operand_stack, &mut execution_stack).is_err());

        operand_stack.push(Object::Integer(1));
        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(3));
        operand_stack.push(Object::Integer(2));

        assert!(index(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(Some(Object::Integer(1)), operand_stack.pop());

        operand_stack.push(Object::Integer(1));
        assert!(index(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(Some(Object::Integer(2)), operand_stack.pop());

        operand_stack.push(Object::Integer(0));
        assert!(index(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(Some(Object::Integer(3)), operand_stack.pop());
    }

    #[test]
    fn test_mark() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        assert!(mark(&mut operand_stack, &mut execution_stack).is_ok());
        assert!(match operand_stack.top() {
            Some(Object::Mark) => true,
            _ => false,
        });
    }

    #[test]
    fn test_clear() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        operand_stack.push(Object::Integer(1));
        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(3));

        assert_eq!(3, operand_stack.count());
        assert!(clear(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(0, operand_stack.count());
    }

    #[test]
    fn test_count() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        assert!(count(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(Some(Object::Integer(0)), operand_stack.top().cloned());

        operand_stack.push(Object::Integer(1));
        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(3));

        assert!(count(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(Some(Object::Integer(4)), operand_stack.top().cloned());
        assert_eq!(5, operand_stack.count());
    }

    #[test]
    fn test_counttomark() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        operand_stack.push(Object::Integer(1));
        operand_stack.push(Object::Mark);
        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(3));

        assert!(counttomark(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(Some(Object::Integer(2)), operand_stack.top().cloned());

        assert!(counttomark(&mut operand_stack, &mut execution_stack).is_ok());

        // Stack should be: 3 2 3 2 MARK 1 |
        assert_eq!(Some(Object::Integer(3)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(3)), operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), operand_stack.pop());
        assert!(match operand_stack.pop() {
            Some(Object::Mark) => true,
            _ => false,
        });
        assert_eq!(Some(Object::Integer(1)), operand_stack.pop());
    }

    #[test]
    fn test_cleartomark() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        operand_stack.push(Object::Integer(1));
        operand_stack.push(Object::Mark);
        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(3));

        assert!(cleartomark(&mut operand_stack, &mut execution_stack).is_ok());
        assert_eq!(1, operand_stack.count());
        assert_eq!(Some(Object::Integer(1)), operand_stack.pop());
    }

    #[test]
    fn test_add() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(1), Object::Integer(1), Object::Integer(2)),
            (Object::Integer(1), Object::Real(1.5), Object::Real(2.5)),
            (Object::Real(1.5), Object::Integer(1), Object::Real(2.5)),
            (Object::Real(1.5), Object::Real(1.5), Object::Real(3.0)),
            (
                Object::Integer(i32::MAX),
                Object::Integer(1),
                Object::Real(f64::from(i32::MAX) + 1.0),
            ),
        ];

        for (lhs, rhs, expect) in cases {
            operand_stack.push(lhs);
            operand_stack.push(rhs);

            assert!(add(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(expect), operand_stack.pop());
        }

        assert!(add(&mut operand_stack, &mut execution_stack)
            .is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

        operand_stack.push(Object::Real(f64::MAX));
        operand_stack.push(Object::Real(f64::MAX));

        assert!(add(&mut operand_stack, &mut execution_stack)
            .is_err_and(|e| e.kind() == ErrorKind::UndefinedResult));
    }

    #[test]
    fn test_div() {
        const ERROR_MARGIN: f64 = 0.000005;

        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(2), Object::Integer(2), 1.0),
            (Object::Integer(1), Object::Integer(3), 0.333333),
            (Object::Integer(-1), Object::Integer(3), -0.333333),
            (Object::Integer(1), Object::Real(0.5), 2.0),
        ];

        for (lhs, rhs, expect) in cases {
            operand_stack.push(lhs);
            operand_stack.push(rhs);

            assert!(div(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert!(if let Some(Object::Real(value)) = operand_stack.pop() {
                (value - expect).abs() < ERROR_MARGIN
            } else {
                false
            });
        }

        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(0));
        assert!(div(&mut operand_stack, &mut execution_stack)
            .is_err_and(|e| e.kind() == ErrorKind::UndefinedResult));
    }

    #[test]
    fn test_idiv() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(1), Object::Integer(2), 0),
            (Object::Integer(10), Object::Integer(3), 3),
            (Object::Integer(5), Object::Integer(2), 2),
        ];

        for (lhs, rhs, expect) in cases {
            operand_stack.push(lhs);
            operand_stack.push(rhs);

            assert!(idiv(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(Object::Integer(expect)), operand_stack.pop());
        }

        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(0));
        assert!(idiv(&mut operand_stack, &mut execution_stack)
            .is_err_and(|e| e.kind() == ErrorKind::UndefinedResult));
    }

    #[test]
    fn test_imod() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(13), Object::Integer(2), 1),
            (Object::Integer(16), Object::Integer(3), 1),
        ];

        for (lhs, rhs, expect) in cases {
            operand_stack.push(lhs);
            operand_stack.push(rhs);

            assert!(imod(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(Object::Integer(expect)), operand_stack.pop());
        }

        operand_stack.push(Object::Integer(2));
        operand_stack.push(Object::Integer(0));
        assert!(imod(&mut operand_stack, &mut execution_stack)
            .is_err_and(|e| e.kind() == ErrorKind::UndefinedResult));
    }

    #[test]
    fn test_mul() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(2), Object::Integer(2), Object::Integer(4)),
            (Object::Integer(2), Object::Real(2.5), Object::Real(5.0)),
            (Object::Real(2.5), Object::Integer(2), Object::Real(5.0)),
            (Object::Real(2.5), Object::Real(2.5), Object::Real(6.25)),
            (
                Object::Integer(i32::MAX),
                Object::Integer(2),
                Object::Real(f64::from(i32::MAX) * 2.0),
            ),
        ];

        for (lhs, rhs, expect) in cases {
            operand_stack.push(lhs);
            operand_stack.push(rhs);

            assert!(mul(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(expect), operand_stack.pop());
        }
    }

    #[test]
    fn test_sub() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(2), Object::Integer(2), Object::Integer(0)),
            (Object::Integer(2), Object::Real(2.5), Object::Real(-0.5)),
            (Object::Real(2.5), Object::Integer(2), Object::Real(0.5)),
            (Object::Real(2.5), Object::Real(2.5), Object::Real(0.0)),
            (
                Object::Integer(i32::MIN),
                Object::Integer(1),
                Object::Real(f64::from(i32::MIN) - 1.0),
            ),
        ];

        for (lhs, rhs, expect) in cases {
            operand_stack.push(lhs);
            operand_stack.push(rhs);

            assert!(sub(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(expect), operand_stack.pop());
        }
    }

    #[test]
    fn test_abs() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(1), Object::Integer(1)),
            (Object::Integer(-1), Object::Integer(1)),
            (Object::Integer(i32::MAX), Object::Integer(i32::MAX)),
            (
                Object::Integer(i32::MIN),
                Object::Real(-1.0 * f64::from(i32::MIN)),
            ),
            (Object::Real(1.0), Object::Real(1.0)),
            (Object::Real(-1.0), Object::Real(1.0)),
            (Object::Real(f64::MAX), Object::Real(f64::MAX)),
            (Object::Real(f64::MIN), Object::Real(f64::MAX)),
        ];

        for (input, expect) in cases {
            operand_stack.push(input);

            assert!(abs(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(expect), operand_stack.pop());
        }
    }
}
