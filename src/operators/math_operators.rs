use std::f64::consts;

use crate::{execution::ExecutionState, object::Object, Error, ErrorKind};

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

/// Returns the sum of num1 and num2. If both operands are integers and the result is
/// within integer range, the result is an integer; otherwise, the result is a real number.
///
/// Errors: StackUnderflow, TypeCheck, UndefinedResult
pub fn add(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let rhs = execution_state.operand_stack.pop();
    let lhs = execution_state.operand_stack.pop();

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => {
                match lhs.checked_add(rhs) {
                    Some(total) => execution_state.operand_stack.push(Object::Integer(total)),
                    None => {
                        let total = f64::from(lhs) + f64::from(rhs);
                        if !is_valid_real(total) {
                            return Err(Error::from(ErrorKind::UndefinedResult));
                        }
                        execution_state.operand_stack.push(Object::Real(total));
                    }
                }
                Ok(())
            }
            (Object::Integer(lhs), Object::Real(rhs)) => {
                let total = f64::from(lhs) + rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Integer(rhs)) => {
                let total = lhs + f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Real(rhs)) => {
                let total = lhs + rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
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
pub fn div(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let rhs = execution_state.operand_stack.pop();
    let lhs = execution_state.operand_stack.pop();

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => {
                let total = f64::from(lhs) / f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Integer(lhs), Object::Real(rhs)) => {
                let total = f64::from(lhs) / rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Integer(rhs)) => {
                let total = lhs / f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Real(rhs)) => {
                let total = lhs / rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
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
pub fn idiv(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let rhs = execution_state.operand_stack.pop();
    let lhs = execution_state.operand_stack.pop();

    match (lhs, rhs) {
        (Some(Object::Integer(lhs)), Some(Object::Integer(rhs))) => match lhs.checked_div(rhs) {
            None => Err(Error::new(ErrorKind::UndefinedResult, "division by zero")),
            Some(total) => {
                execution_state.operand_stack.push(Object::Integer(total));
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
pub fn imod(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let rhs = execution_state.operand_stack.pop();
    let lhs = execution_state.operand_stack.pop();

    match (lhs, rhs) {
        (Some(Object::Integer(lhs)), Some(Object::Integer(rhs))) => match lhs.checked_rem(rhs) {
            None => Err(Error::new(ErrorKind::UndefinedResult, "division by zero")),
            Some(total) => {
                execution_state.operand_stack.push(Object::Integer(total));
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
pub fn mul(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let rhs = execution_state.operand_stack.pop();
    let lhs = execution_state.operand_stack.pop();

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => {
                match lhs.checked_mul(rhs) {
                    Some(total) => execution_state.operand_stack.push(Object::Integer(total)),
                    None => {
                        let total = f64::from(lhs) * f64::from(rhs);
                        if !is_valid_real(total) {
                            return Err(Error::from(ErrorKind::UndefinedResult));
                        }
                        execution_state.operand_stack.push(Object::Real(total));
                    }
                }
                Ok(())
            }
            (Object::Integer(lhs), Object::Real(rhs)) => {
                let total = f64::from(lhs) * rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Integer(rhs)) => {
                let total = lhs * f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Real(rhs)) => {
                let total = lhs * rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
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
pub fn sub(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let rhs = execution_state.operand_stack.pop();
    let lhs = execution_state.operand_stack.pop();

    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => match (lhs, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => {
                match lhs.checked_sub(rhs) {
                    Some(total) => execution_state.operand_stack.push(Object::Integer(total)),
                    None => {
                        let total = f64::from(lhs) - f64::from(rhs);
                        if !is_valid_real(total) {
                            return Err(Error::from(ErrorKind::UndefinedResult));
                        }
                        execution_state.operand_stack.push(Object::Real(total));
                    }
                }
                Ok(())
            }
            (Object::Integer(lhs), Object::Real(rhs)) => {
                let total = f64::from(lhs) - rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Integer(rhs)) => {
                let total = lhs - f64::from(rhs);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(lhs), Object::Real(rhs)) => {
                let total = lhs - rhs;
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
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
pub fn abs(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            match value.checked_abs() {
                Some(total) => execution_state.operand_stack.push(Object::Integer(total)),
                None => {
                    let total = f64::from(value).abs();
                    if !is_valid_real(total) {
                        return Err(Error::from(ErrorKind::UndefinedResult));
                    }
                    execution_state.operand_stack.push(Object::Real(total));
                }
            }
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.abs();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
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
pub fn neg(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            match value.checked_mul(-1) {
                Some(total) => execution_state.operand_stack.push(Object::Integer(total)),
                None => {
                    let total = -1.0 * f64::from(value);
                    if !is_valid_real(total) {
                        return Err(Error::from(ErrorKind::UndefinedResult));
                    }
                    execution_state.operand_stack.push(Object::Real(total));
                }
            }
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = -1.0 * value;
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
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
pub fn ceiling(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            execution_state.operand_stack.push(Object::Integer(value));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.ceil();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
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
pub fn floor(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            execution_state.operand_stack.push(Object::Integer(value));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.floor();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
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
pub fn round(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            execution_state.operand_stack.push(Object::Integer(value));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.round();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
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
pub fn truncate(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            execution_state.operand_stack.push(Object::Integer(value));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.trunc();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
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
pub fn sqrt(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            let total = f64::from(value).sqrt();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.sqrt();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
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
pub fn atan(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let den = execution_state.operand_stack.pop();
    let num = execution_state.operand_stack.pop();

    match (num, den) {
        (Some(num), Some(den)) => match (num, den) {
            (Object::Integer(num), Object::Integer(den)) => {
                let total =
                    positive_degrees(radians_to_degrees((f64::from(num) / f64::from(den)).atan()));
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Integer(num), Object::Real(den)) => {
                let total = positive_degrees(radians_to_degrees((f64::from(num) / den).atan()));
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(num), Object::Integer(den)) => {
                let total = positive_degrees(radians_to_degrees((num / f64::from(den)).atan()));
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(num), Object::Real(den)) => {
                let total = positive_degrees(radians_to_degrees((num / den).atan()));
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
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
pub fn cos(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            let total = degrees_to_radians(f64::from(value)).cos();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = degrees_to_radians(value).cos();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
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
pub fn sin(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            let total = degrees_to_radians(f64::from(value)).sin();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = degrees_to_radians(value).sin();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected numeric")),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Raises base to the exponent power. The operands may be either integers or real
/// numbers. If the exponent has a fractional part, the result is meaningful only if the
/// base is nonnegative. The result is always a real number.
///
/// Errors: StackUnderflow, TypeCheck, UndefinedResult
pub fn exp(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let exponent = execution_state.operand_stack.pop();
    let base = execution_state.operand_stack.pop();

    match (base, exponent) {
        (Some(base), Some(exponent)) => match (base, exponent) {
            (Object::Integer(base), Object::Integer(exponent)) => {
                let total = f64::from(base).powi(exponent);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Integer(base), Object::Real(exponent)) => {
                let total = f64::from(base).powf(exponent);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(base), Object::Integer(exponent)) => {
                let total = base.powi(exponent);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            (Object::Real(base), Object::Real(exponent)) => {
                let total = base.powf(exponent);
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }
                execution_state.operand_stack.push(Object::Real(total));
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected numerics")),
        },
        _ => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}
/// Returns the natural logarithm (base e) of num. The result is a real number.
///
/// Errors: RangeCheck, StackUnderflow, TypeCheck
pub fn ln(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            let total = f64::from(value).ln();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.ln();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected numeric")),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Returns the common logarithm (base 10) of num. The result is a real number.
///
/// Errors: RangeCheck, StackUnderflow, TypeCheck
pub fn log(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            let total = f64::from(value).log10();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(Object::Real(value)) => {
            let total = value.log10();
            if !is_valid_real(total) {
                return Err(Error::from(ErrorKind::UndefinedResult));
            }
            execution_state.operand_stack.push(Object::Real(total));
            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected numeric")),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Returns a random integer in the range 0 to 2^31 − 1, produced by a pseudo-random
/// number generator. The random number generator’s state can be reset by srand
/// and interrogated by rrand.
///
/// Errors: StackOverflow
pub fn rand(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let num = execution_state.rng.rand().abs();
    execution_state.operand_stack.push(Object::Integer(num));

    Ok(())
}

/// Initializes the random number generator with the seed int, which may be any inte-
/// ger value. Executing srand with a particular value causes subsequent invocations
/// of rand to generate a reproducible sequence of results. In an interpreter that sup-
/// ports multiple execution contexts, the random number state is maintained sepa-
/// rately for each context.
///
/// Errors: StackUnderflow, TypeCheck
pub fn srand(execution_state: &mut ExecutionState) -> crate::Result<()> {
    match execution_state.operand_stack.pop() {
        Some(Object::Integer(value)) => {
            execution_state.rng.set_seed(value);
            Ok(())
        }
        Some(_) => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        None => Err(Error::from(ErrorKind::StackUnderflow)),
    }
}

/// Returns an integer representing the current state of the random number generator
/// used by rand. This may later be presented as an operand to srand to reset the
/// random number generator to the current position in the sequence of numbers
/// produced.
///
/// Errors: StackOverflow
pub fn rrand(execution_state: &mut ExecutionState) -> crate::Result<()> {
    let seed = execution_state.rng.seed();
    execution_state.operand_stack.push(Object::Integer(seed));

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    const ERROR_MARGIN: f64 = 0.000005;

    #[test]
    fn test_add() {
        let mut execution_state = ExecutionState::new();

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
            execution_state.operand_stack.push(lhs);
            execution_state.operand_stack.push(rhs);

            assert!(add(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(Some(expect), execution_state.operand_stack.pop());
        }

        assert!(add(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

        execution_state.operand_stack.push(Object::Real(f64::MAX));
        execution_state.operand_stack.push(Object::Real(f64::MAX));

        assert!(add(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::UndefinedResult));
    }

    #[test]
    fn test_div() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(2), Object::Integer(2), 1.0),
            (Object::Integer(1), Object::Integer(3), 0.333333),
            (Object::Integer(-1), Object::Integer(3), -0.333333),
            (Object::Integer(1), Object::Real(0.5), 2.0),
        ];

        for (lhs, rhs, expect) in cases {
            execution_state.operand_stack.push(lhs);
            execution_state.operand_stack.push(rhs);

            assert!(div(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect).abs() < ERROR_MARGIN);
        }

        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(0));
        assert!(div(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::UndefinedResult));

        Ok(())
    }

    #[test]
    fn test_idiv() {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(1), Object::Integer(2), 0),
            (Object::Integer(10), Object::Integer(3), 3),
            (Object::Integer(5), Object::Integer(2), 2),
        ];

        for (lhs, rhs, expect) in cases {
            execution_state.operand_stack.push(lhs);
            execution_state.operand_stack.push(rhs);

            assert!(idiv(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(
                Some(Object::Integer(expect)),
                execution_state.operand_stack.pop()
            );
        }

        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(0));
        assert!(idiv(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::UndefinedResult));
    }

    #[test]
    fn test_imod() {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(13), Object::Integer(2), 1),
            (Object::Integer(16), Object::Integer(3), 1),
        ];

        for (lhs, rhs, expect) in cases {
            execution_state.operand_stack.push(lhs);
            execution_state.operand_stack.push(rhs);

            assert!(imod(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(
                Some(Object::Integer(expect)),
                execution_state.operand_stack.pop()
            );
        }

        execution_state.operand_stack.push(Object::Integer(2));
        execution_state.operand_stack.push(Object::Integer(0));
        assert!(imod(&mut execution_state).is_err_and(|e| e.kind() == ErrorKind::UndefinedResult));
    }

    #[test]
    fn test_mul() {
        let mut execution_state = ExecutionState::new();

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
            execution_state.operand_stack.push(lhs);
            execution_state.operand_stack.push(rhs);

            assert!(mul(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(Some(expect), execution_state.operand_stack.pop());
        }
    }

    #[test]
    fn test_sub() {
        let mut execution_state = ExecutionState::new();

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
            execution_state.operand_stack.push(lhs);
            execution_state.operand_stack.push(rhs);

            assert!(sub(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(Some(expect), execution_state.operand_stack.pop());
        }
    }

    #[test]
    fn test_abs() {
        let mut execution_state = ExecutionState::new();

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
            execution_state.operand_stack.push(input);

            assert!(abs(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(Some(expect), execution_state.operand_stack.pop());
        }
    }

    #[test]
    fn test_neg() {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(-1), Object::Integer(1)),
            (Object::Integer(1), Object::Integer(-1)),
            (Object::Real(-1.0), Object::Real(1.0)),
            (Object::Real(1.0), Object::Real(-1.0)),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(neg(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(Some(expect), execution_state.operand_stack.pop());
        }
    }

    #[test]
    fn test_ceiling() {
        let mut execution_state = ExecutionState::new();

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
            execution_state.operand_stack.push(input);

            assert!(ceiling(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(Some(expect), execution_state.operand_stack.pop());
        }
    }

    #[test]
    fn test_floor() {
        let mut execution_state = ExecutionState::new();

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
            execution_state.operand_stack.push(input);

            assert!(floor(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(Some(expect), execution_state.operand_stack.pop());
        }
    }

    #[test]
    fn test_round() {
        let mut execution_state = ExecutionState::new();

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
            execution_state.operand_stack.push(input);

            assert!(round(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(Some(expect), execution_state.operand_stack.pop());
        }
    }

    #[test]
    fn test_truncate() {
        let mut execution_state = ExecutionState::new();

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
            execution_state.operand_stack.push(input);

            assert!(truncate(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(Some(expect), execution_state.operand_stack.pop());
        }
    }

    #[test]
    fn test_sqrt() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(1), 1.0),
            (Object::Real(1.0), 1.0),
            (Object::Integer(2), 1.414213),
            (Object::Real(2.0), 1.414213),
            (Object::Integer(4), 2.0),
            (Object::Real(4.0), 2.0),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(sqrt(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_atan() {
        let mut execution_state = ExecutionState::new();

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
            execution_state.operand_stack.push(num);
            execution_state.operand_stack.push(den);

            assert!(atan(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(
                Some(Object::Real(expect)),
                execution_state.operand_stack.pop()
            );
        }
    }

    #[test]
    fn test_cos() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(0), 1.0),
            (Object::Real(0.0), 1.0),
            (Object::Integer(90), 0.0),
            (Object::Real(-90.0), 0.0),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(cos(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_sin() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(0), 0.0),
            (Object::Real(0.0), 0.0),
            (Object::Integer(90), 1.0),
            (Object::Real(-90.0), -1.0),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(sin(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_exp() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(1), Object::Integer(100), 1.0),
            (Object::Real(1.0), Object::Real(100.0), 1.0),
            (Object::Integer(2), Object::Integer(8), 256.0),
            (Object::Real(2.0), Object::Real(8.0), 256.0),
            (Object::Real(2.0), Object::Real(8.0), 256.0),
            (Object::Real(2.0), Object::Real(8.0), 256.0),
            (Object::Integer(8), Object::Real(1.0 / 3.0), 2.0),
            (Object::Real(8.0), Object::Real(1.0 / 3.0), 2.0),
            (Object::Integer(9), Object::Real(0.5), 3.0),
            (Object::Real(9.0), Object::Real(0.5), 3.0),
            (Object::Integer(-9), Object::Integer(-1), -0.111111),
            (Object::Real(-9.0), Object::Real(-1.0), -0.111111),
        ];

        for (base, exponent, expect) in cases {
            execution_state.operand_stack.push(base);
            execution_state.operand_stack.push(exponent);

            assert!(exp(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect) < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_ln() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(10), 2.302585),
            (Object::Real(10.0), 2.302585),
            (Object::Integer(100), 4.60517),
            (Object::Real(100.0), 4.60517),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(ln(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect) < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_log() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(10), 1.0),
            (Object::Real(10.0), 1.0),
            (Object::Integer(100), 2.0),
            (Object::Real(100.0), 2.0),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(log(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect) < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_rand() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        for _ in 0..5 {
            assert!(rand(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Integer(value)) = execution_state.operand_stack.pop() else {
                return Err("expected integer object".into());
            };
            assert!(value >= 0);
        }

        Ok(())
    }

    #[test]
    fn test_srand() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(100));
        assert!(srand(&mut execution_state).is_ok());
        assert!(rand(&mut execution_state).is_ok());
        let Some(Object::Integer(initial_value)) = execution_state.operand_stack.pop() else {
            return Err("expected integer object".into());
        };

        execution_state.operand_stack.push(Object::Integer(1));
        assert!(srand(&mut execution_state).is_ok());
        assert!(rand(&mut execution_state).is_ok());
        let Some(Object::Integer(intmd_value)) = execution_state.operand_stack.pop() else {
            return Err("expected integer object".into());
        };
        assert_ne!(initial_value, intmd_value);

        execution_state.operand_stack.push(Object::Integer(100));
        assert!(srand(&mut execution_state).is_ok());
        assert!(rand(&mut execution_state).is_ok());
        let Some(Object::Integer(end_value)) = execution_state.operand_stack.pop() else {
            return Err("expected integer object".into());
        };
        assert_eq!(initial_value, end_value);

        Ok(())
    }

    #[test]
    fn test_rrand() {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(1));
        assert!(srand(&mut execution_state).is_ok());
        for _ in 0..3 {
            assert!(rand(&mut execution_state).is_ok());
        }
        assert!(rrand(&mut execution_state).is_ok());
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop(),
        );
        execution_state.operand_stack.clear();
    }
}
