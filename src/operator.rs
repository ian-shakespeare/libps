use std::f64::consts;

use crate::{object::Object, stack::Stack, Error, ErrorKind};

fn is_valid_real(real: f64) -> bool {
    real.is_finite() && !real.is_nan()
}

fn radians_to_degrees(radians: f64) -> f64 {
    radians * (180.0 / consts::PI)
}

fn degrees_to_radians(degrees: f64) -> f64 {
    (degrees * consts::PI) / 180.0
}

fn positive_degrees(degrees: f64) -> f64 {
    if degrees < 0.0 {
        360.0 + degrees
    } else {
        degrees
    }
}

/// Duplicates the top element on the operand stack. dup copies only the object; the
/// value of a composite object is not copied but is shared. See Section 3.3, “Data
/// Types and Objects.”
///
/// Errors: StackOverflow, StackUnderflow
pub fn dup(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.top() {
        None => Err(Error::from(ErrorKind::StackUnderflow)),
        Some(top) => {
            operand_stack.push(top.clone());
            Ok(())
        }
    }
}

/// Exchanges the top two elements on the operand stack.
///
/// Errors: StackUnderflow
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

/// Removes the top element from the operand stack and discards it.
///
/// Errors: StackUnderflow
pub fn pop(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    if operand_stack.pop().is_none() {
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

/// Removes the nonnegative integer n from the operand stack, counts down to the
/// nth element from the top of the stack, and pushes a copy of that element on the
/// stack.
///
/// Errors: RangeCheck, StackUnderflow, TypeCheck
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
pub fn mark(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    operand_stack.push(Object::Mark);

    Ok(())
}

/// Pops all objects from the operand stack and discards them.
///
/// Errors: None
pub fn clear(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    operand_stack.clear();

    Ok(())
}

/// Counts the number of items on the operand stack and pushes this count on the
/// operand stack.
///
/// Errors: StackOverflow
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

/// Counts the number of objects on the operand stack, starting with the top element
/// and continuing down to but not including the first mark encountered. obj1
/// through obj n are any objects other than marks.
///
/// Errors: StackOverflow, UnmatchedMark
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

/// Pops entries from the operand stack repeatedly until it encounters a mark, which
/// it also pops from the stack. obj1 through obj n are any objects other than marks.
///
/// Errors: UnmatchedMark
pub fn cleartomark(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    while let Some(obj) = operand_stack.pop() {
        if let Object::Mark = obj {
            break;
        }
    }

    Ok(())
}

/// Returns the sum of num1 and num2. If both operands are integers and the result is
/// within integer range, the result is an integer; otherwise, the result is a real number.
///
/// Errors: StackUnderflow, TypeCheck, UndefinedResult
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

/// Divides num1 by num2, producing a result that is always a real number even if both
/// operands are integers. Use idiv instead if the operands are integers and an integer
/// result is desired.
///
/// Errors: StackUnderflow, TypeCheck, UndefinedResult
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

/// Divides int1 by int2 and returns the integer part of the quotient, with any fractional
/// part discarded. Both operands of idiv must be integers and the result is an integer.
///
/// Errors: StackUnderflow, TypeCheck, UndefinedResult
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

/// Returns the remainder that results from dividing int1 by int2. The sign of the result
/// is the same as the sign of the dividend int1. Both operands must be integers and
/// the result is an integer.
///
/// Errors: StackUnderflow, TypeCheck, UndefinedResult
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

/// Returns the product of num1 and num2. If both operands are integers and the re-
/// sult is within integer range, the result is an integer; otherwise, the result is a real
/// number.
///
/// Errors: StackUnderflow, TypeCheck, UndefinedResult
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

/// Returns the result of subtracting num2 from num1. If both operands are integers
/// and the result is within integer range, the result is an integer; otherwise, the result
/// is a real number.
///
/// Errors: StackUnderflow, TypeCheck, UndefinedResult
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

/// Returns the absolute value of num1. The type of the result is the same as the type of
/// num1 unless num1 is the smallest (most negative) integer, in which case the result
/// is a real number.
///
/// Errors: StackUnderflow, TypeCheck
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

/// Returns the negative of num1. The type of the result is the same as the type of num1
/// unless num1 is the smallest (most negative) integer, in which case the result is a
/// real number.
///
/// Errors: StackUnderflow, TypeCheck
pub fn neg(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.pop() {
        Some(Object::Integer(value)) => {
            match value.checked_mul(-1) {
                Some(total) => operand_stack.push(Object::Integer(total)),
                None => {
                    let total = -1.0 * f64::from(value);
                    if !is_valid_real(total) {
                        return Err(Error::from(ErrorKind::UndefinedResult));
                    }
                    operand_stack.push(Object::Real(total));
                }
            }
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = -1.0 * value;
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

/// Returns the least integer value greater than or equal to num1. The type of the result
/// is the same as the type of the operand.
///
/// Errors: StackUnderflow, TypeCheck
pub fn ceiling(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.pop() {
        Some(Object::Integer(value)) => {
            operand_stack.push(Object::Integer(value));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.ceil();
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

/// Returns the greatest integer value less than or equal to num1. The type of the result
/// is the same as the type of the operand.
///
/// Errors: StackUnderflow, TypeCheck
pub fn floor(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.pop() {
        Some(Object::Integer(value)) => {
            operand_stack.push(Object::Integer(value));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.floor();
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

/// Returns the integer value nearest to num1. If num1 is equally close to its two nearest
/// integers, round returns the greater of the two. The type of the result is the same as
/// the type of the operand.
///
/// Errors: StackUnderflow, TypeCheck
pub fn round(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.pop() {
        Some(Object::Integer(value)) => {
            operand_stack.push(Object::Integer(value));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.round();
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

/// Truncates num1 toward 0 by removing its fractional part. The type of the result is
/// the same as the type of the operand.
///
/// Errors: StackUnderflow, TypeCheck
pub fn truncate(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.pop() {
        Some(Object::Integer(value)) => {
            operand_stack.push(Object::Integer(value));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.trunc();
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

/// Returns the square root of num, which must be a nonnegative number. The result
/// is a real number.
///
/// Errors: RangeCheck, StackUnderflow, TypeCheck
pub fn sqrt(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.pop() {
        Some(Object::Integer(value)) => {
            let total = f64::from(value).sqrt();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.sqrt();
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
/// Returns the angle (in degrees between 0 and 360) whose tangent is num divided by
/// den. Either num or den may be 0, but not both. The signs of num and den deter-
/// mine the quadrant in which the result will lie: a positive num yields a result in the
/// positive y plane, while a positive den yields a result in the positive x plane. The re-
/// sult is a real number.
///
/// Errors: StackUnderflow, TypeCheck, UndefinedResult
pub fn atan(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    let den = operand_stack.pop();
    let num = operand_stack.pop();

    match (num, den) {
        (Some(num), Some(den)) => match (num, den) {
            (Object::Integer(num), Object::Integer(den)) => {
                let total =
                    positive_degrees(radians_to_degrees((f64::from(num) / f64::from(den)).atan()));
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Integer(num), Object::Real(den)) => {
                let total = positive_degrees(radians_to_degrees((f64::from(num) / den).atan()));
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(num), Object::Integer(den)) => {
                let total = positive_degrees(radians_to_degrees((num / f64::from(den)).atan()));
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(num), Object::Real(den)) => {
                let total = positive_degrees(radians_to_degrees((num / den).atan()));
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

/// Returns the cosine of angle, which is interpreted as an angle in degrees. The result
/// is a real number.
///
/// Errors: StackUnderflow, TypeCheck
pub fn cos(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.pop() {
        Some(Object::Integer(value)) => {
            let total = degrees_to_radians(f64::from(value)).cos();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = degrees_to_radians(value).cos();
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

/// Returns the sine of angle, which is interpreted as an angle in degrees. The result is a
/// real number.
///
/// Errors: StackUnderflow, TypeCheck
pub fn sin(operand_stack: &mut Stack<Object>, _: &mut Stack<Object>) -> crate::Result<()> {
    match operand_stack.pop() {
        Some(Object::Integer(value)) => {
            let total = degrees_to_radians(f64::from(value)).sin();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = degrees_to_radians(value).sin();
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

    const ERROR_MARGIN: f64 = 0.000005;

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

    #[test]
    fn test_neg() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(-1), Object::Integer(1)),
            (Object::Integer(1), Object::Integer(-1)),
            (Object::Real(-1.0), Object::Real(1.0)),
            (Object::Real(1.0), Object::Real(-1.0)),
        ];

        for (input, expect) in cases {
            operand_stack.push(input);

            assert!(neg(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(expect), operand_stack.pop());
        }
    }

    #[test]
    fn test_ceiling() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(1), Object::Integer(1)),
            (Object::Real(1.0), Object::Real(1.0)),
            (Object::Real(1.0000001), Object::Real(2.0)),
            (Object::Real(1.1), Object::Real(2.0)),
            (Object::Real(1.2), Object::Real(2.0)),
            (Object::Real(1.3), Object::Real(2.0)),
            (Object::Real(1.4), Object::Real(2.0)),
            (Object::Real(1.5), Object::Real(2.0)),
            (Object::Real(1.6), Object::Real(2.0)),
            (Object::Real(1.7), Object::Real(2.0)),
            (Object::Real(1.8), Object::Real(2.0)),
            (Object::Real(1.9), Object::Real(2.0)),
            (Object::Real(1.9999999), Object::Real(2.0)),
        ];

        for (input, expect) in cases {
            operand_stack.push(input);

            assert!(ceiling(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(expect), operand_stack.pop());
        }
    }

    #[test]
    fn test_floor() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(1), Object::Integer(1)),
            (Object::Real(1.0), Object::Real(1.0)),
            (Object::Real(1.0000001), Object::Real(1.0)),
            (Object::Real(1.1), Object::Real(1.0)),
            (Object::Real(1.2), Object::Real(1.0)),
            (Object::Real(1.3), Object::Real(1.0)),
            (Object::Real(1.4), Object::Real(1.0)),
            (Object::Real(1.5), Object::Real(1.0)),
            (Object::Real(1.6), Object::Real(1.0)),
            (Object::Real(1.7), Object::Real(1.0)),
            (Object::Real(1.8), Object::Real(1.0)),
            (Object::Real(1.9), Object::Real(1.0)),
            (Object::Real(1.9999999), Object::Real(1.0)),
        ];

        for (input, expect) in cases {
            operand_stack.push(input);

            assert!(floor(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(expect), operand_stack.pop());
        }
    }

    #[test]
    fn test_round() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(1), Object::Integer(1)),
            (Object::Real(1.0), Object::Real(1.0)),
            (Object::Real(1.0000001), Object::Real(1.0)),
            (Object::Real(1.1), Object::Real(1.0)),
            (Object::Real(1.2), Object::Real(1.0)),
            (Object::Real(1.3), Object::Real(1.0)),
            (Object::Real(1.4), Object::Real(1.0)),
            (Object::Real(1.4999999), Object::Real(1.0)),
            (Object::Real(1.5), Object::Real(2.0)),
            (Object::Real(1.6), Object::Real(2.0)),
            (Object::Real(1.7), Object::Real(2.0)),
            (Object::Real(1.8), Object::Real(2.0)),
            (Object::Real(1.9), Object::Real(2.0)),
            (Object::Real(1.9999999), Object::Real(2.0)),
        ];

        for (input, expect) in cases {
            operand_stack.push(input);

            assert!(round(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(expect), operand_stack.pop());
        }
    }

    #[test]
    fn test_truncate() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(1), Object::Integer(1)),
            (Object::Real(1.0), Object::Real(1.0)),
            (Object::Real(1.0000001), Object::Real(1.0)),
            (Object::Real(1.1), Object::Real(1.0)),
            (Object::Real(1.2), Object::Real(1.0)),
            (Object::Real(1.3), Object::Real(1.0)),
            (Object::Real(1.4), Object::Real(1.0)),
            (Object::Real(1.5), Object::Real(1.0)),
            (Object::Real(1.6), Object::Real(1.0)),
            (Object::Real(1.7), Object::Real(1.0)),
            (Object::Real(1.8), Object::Real(1.0)),
            (Object::Real(1.9), Object::Real(1.0)),
            (Object::Real(1.9999999), Object::Real(1.0)),
        ];

        for (input, expect) in cases {
            operand_stack.push(input);

            assert!(truncate(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(expect), operand_stack.pop());
        }
    }

    #[test]
    fn test_sqrt() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(1), 1.0),
            (Object::Real(1.0), 1.0),
            (Object::Integer(2), 1.414213),
            (Object::Real(2.0), 1.414213),
            (Object::Integer(4), 2.0),
            (Object::Real(4.0), 2.0),
        ];

        for (input, expect) in cases {
            operand_stack.push(input);

            assert!(sqrt(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert!(if let Some(Object::Real(value)) = operand_stack.pop() {
                (value - expect).abs() < ERROR_MARGIN
            } else {
                false
            });
        }
    }

    #[test]
    fn test_atan() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(0), Object::Integer(1), 0.0),
            (Object::Real(0.0), Object::Real(1.0), 0.0),
            (Object::Integer(1), Object::Integer(0), 90.0),
            (Object::Real(1.0), Object::Real(0.0), 90.0),
            (Object::Integer(-100), Object::Integer(0), 270.0),
            (Object::Real(-100.0), Object::Real(0.0), 270.0),
            (Object::Integer(4), Object::Integer(4), 45.0),
            (Object::Real(4.0), Object::Real(4.0), 45.0),
        ];

        for (num, den, expect) in cases {
            operand_stack.push(num);
            operand_stack.push(den);

            assert!(atan(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert_eq!(Some(Object::Real(expect)), operand_stack.pop());
        }
    }

    #[test]
    fn test_cos() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(0), 1.0),
            (Object::Real(0.0), 1.0),
            (Object::Integer(90), 0.0),
            (Object::Real(-90.0), 0.0),
        ];

        for (input, expect) in cases {
            operand_stack.push(input);

            assert!(cos(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert!(if let Some(Object::Real(value)) = operand_stack.pop() {
                (value - expect).abs() < ERROR_MARGIN
            } else {
                false
            });
        }
    }

    #[test]
    fn test_sin() {
        let mut operand_stack: Stack<Object> = Stack::new();
        let mut execution_stack: Stack<Object> = Stack::new();

        let cases = [
            (Object::Integer(0), 0.0),
            (Object::Real(0.0), 0.0),
            (Object::Integer(90), 1.0),
            (Object::Real(-90.0), -1.0),
        ];

        for (input, expect) in cases {
            operand_stack.push(input);

            assert!(sin(&mut operand_stack, &mut execution_stack).is_ok());
            assert_eq!(1, operand_stack.count());
            assert!(if let Some(Object::Real(value)) = operand_stack.pop() {
                (value - expect).abs() < ERROR_MARGIN
            } else {
                false
            });
        }
    }
}
