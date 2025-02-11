use crate::{Error, ErrorKind, Interpreter, Object};

use super::{degrees_to_radians, is_valid_real, positive_degrees, radians_to_degrees};

pub fn arithmetic(
    interpreter: &mut Interpreter,
    checked: impl Fn(i32, i32) -> Option<i32>,
    real: impl Fn(f64, f64) -> f64,
) -> crate::Result<()> {
    let rhs = interpreter.pop()?;
    let lhs = interpreter.pop()?;

    if lhs.is_int() && rhs.is_int() {
        let lhs = lhs.into_int()?;
        let rhs = rhs.into_int()?;

        let obj = match checked(lhs, rhs) {
            Some(total) => Object::Integer(total),
            None => {
                let total = real(f64::from(lhs), f64::from(rhs));
                if !is_valid_real(total) {
                    return Err(Error::from(ErrorKind::UndefinedResult));
                }

                Object::Real(total)
            },
        };

        interpreter.push(obj);

        return Ok(());
    }

    let total = real(lhs.into_real()?, rhs.into_real()?);

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    interpreter.push(Object::Real(total));

    Ok(())
}

pub fn num_unary(
    interpreter: &mut Interpreter,
    int: impl Fn(i32) -> Option<i32>,
    real: impl Fn(f64) -> f64,
) -> crate::Result<()> {
    let n = interpreter.pop()?;

    if n.is_int() {
        let obj = match int(n.into_int()?) {
            Some(total) => Object::Integer(total),
            None => Object::Real(real(n.into_real()?)),
        };

        interpreter.push(obj);

        return Ok(());
    }

    interpreter.push(Object::Real(real(n.into_real()?)));

    Ok(())
}

pub fn real_unary(interpreter: &mut Interpreter, unary: impl Fn(f64) -> f64) -> crate::Result<()> {
    let n = interpreter.pop_real()?;

    let total = unary(n);

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    interpreter.push(Object::Real(total));

    Ok(())
}

pub fn idiv(interpreter: &mut Interpreter) -> crate::Result<()> {
    let rhs = interpreter.pop_int()?;
    let lhs = interpreter.pop_int()?;

    let total = match lhs.checked_div(rhs) {
        Some(total) => Ok(total),
        None => Err(Error::from(ErrorKind::UndefinedResult)),
    }?;

    interpreter.push(Object::Integer(total));

    Ok(())
}

pub fn imod(interpreter: &mut Interpreter) -> crate::Result<()> {
    let rhs = interpreter.pop_int()?;
    let lhs = interpreter.pop_int()?;

    let total = match lhs.checked_rem(rhs) {
        Some(total) => Ok(total),
        None => Err(Error::from(ErrorKind::UndefinedResult)),
    }?;

    interpreter.push(Object::Integer(total));

    Ok(())
}

pub fn atan(interpreter: &mut Interpreter) -> crate::Result<()> {
    let den = interpreter.pop_real()?;
    let num = interpreter.pop_real()?;

    let total = positive_degrees(radians_to_degrees((num / den).atan()));

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    interpreter.push(Object::Real(total));

    Ok(())
}

pub fn cos(interpreter: &mut Interpreter) -> crate::Result<()> {
    let n = interpreter.pop_real()?;

    let total = degrees_to_radians(n).cos();

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    interpreter.push(Object::Real(total));

    Ok(())
}

pub fn sin(interpreter: &mut Interpreter) -> crate::Result<()> {
    let n = interpreter.pop_real()?;

    let total = degrees_to_radians(n).sin();

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    interpreter.push(Object::Real(total));

    Ok(())
}

pub fn rand(interpreter: &mut Interpreter) -> crate::Result<()> {
    let n = interpreter.rng.rand().abs();

    interpreter.push(Object::Integer(n));

    Ok(())
}

pub fn srand(interpreter: &mut Interpreter) -> crate::Result<()> {
    let seed = interpreter.pop_int()?;

    interpreter.rng.set_seed(seed);

    Ok(())
}

pub fn rrand(interpreter: &mut Interpreter) -> crate::Result<()> {
    let seed = interpreter.rng.seed();

    interpreter.push(Object::Integer(seed));

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    const ERROR_MARGIN: f64 = 0.000005;

    #[test]
    fn test_add() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 1 add", Object::Integer(2)),
            ("1 1.5 add", Object::Real(2.5)),
            ("1.5 1 add", Object::Real(2.5)),
            ("1.5 1.5 add", Object::Real(3.0)),
            ("16#7FFFFFFF 1 add", Object::Real(f64::from(i32::MAX) + 1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_add_typecheck() {
        let cases = ["(string) 1.2e7 add", "1.2e7 (string) add"];

        for input in cases {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_add_underflow() {
        let cases = ["1 add", "add"];

        for input in cases {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_div() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("2 2 div", 1.0),
            ("1 3 div", 0.333333),
            ("-1 3 div", -0.333333),
            ("1 0.5 div", 2.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            let Some(Object::Real(value)) = interpreter.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_div_undefinedresult() {
        let cases = ["1 0 div", "1 0.0 div"];

        for input in cases {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::UndefinedResult, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_idiv() -> Result<(), Box<dyn error::Error>> {
        let cases = [("1 2 idiv", 0), ("10 3 idiv", 3), ("5 2 idiv", 2)];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_idiv_undefinedresult() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 0 idiv".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::UndefinedResult, result.unwrap_err().kind());
    }

    #[test]
    fn test_imod() -> Result<(), Box<dyn error::Error>> {
        let cases = [("13 2 imod", 1), ("16 3 imod", 1)];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_imod_undefinedresult() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("2 0 imod".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::UndefinedResult, result.unwrap_err().kind());
    }

    #[test]
    fn test_mul() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("2 2 mul", Object::Integer(4)),
            ("2 2.5 mul", Object::Real(5.0)),
            ("2.5 2 mul", Object::Real(5.0)),
            ("2.5 2.5 mul", Object::Real(6.25)),
            ("16#7FFFFFFF 2 mul", Object::Real(f64::from(i32::MAX) * 2.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_sub() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("2 2 sub", Object::Integer(0)),
            ("2 2.5 sub", Object::Real(-0.5)),
            ("2.5 2 sub", Object::Real(0.5)),
            ("2.5 2.5 sub", Object::Real(0.0)),
            ("-2147483648 1 sub", Object::Real(f64::from(i32::MIN) - 1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_abs() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 abs", Object::Integer(1)),
            ("-1 abs", Object::Integer(1)),
            ("16#7FFFFFFF abs", Object::Integer(i32::MAX)),
            ("-2147483648 abs", Object::Real(-1.0 * f64::from(i32::MIN))),
            ("1.0 abs", Object::Real(1.0)),
            ("-1.0 abs", Object::Real(1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_neg() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("-1 neg", Object::Integer(1)),
            ("1 neg", Object::Integer(-1)),
            ("-1.0 neg", Object::Real(1.0)),
            ("1.0 neg", Object::Real(-1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_ceiling() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 ceiling", Object::Integer(1)),
            ("1.0 ceiling", Object::Real(1.0)),
            ("1.0000001 ceiling", Object::Real(2.0)),
            ("1.1 ceiling", Object::Real(2.0)),
            ("1.2 ceiling", Object::Real(2.0)),
            ("1.3 ceiling", Object::Real(2.0)),
            ("1.4 ceiling", Object::Real(2.0)),
            ("1.5 ceiling", Object::Real(2.0)),
            ("1.6 ceiling", Object::Real(2.0)),
            ("1.7 ceiling", Object::Real(2.0)),
            ("1.8 ceiling", Object::Real(2.0)),
            ("1.9 ceiling", Object::Real(2.0)),
            ("1.9999999 ceiling", Object::Real(2.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_floor() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 floor", Object::Integer(1)),
            ("1.0 floor", Object::Real(1.0)),
            ("1.0000001 floor", Object::Real(1.0)),
            ("1.1 floor", Object::Real(1.0)),
            ("1.2 floor", Object::Real(1.0)),
            ("1.3 floor", Object::Real(1.0)),
            ("1.4 floor", Object::Real(1.0)),
            ("1.5 floor", Object::Real(1.0)),
            ("1.6 floor", Object::Real(1.0)),
            ("1.7 floor", Object::Real(1.0)),
            ("1.8 floor", Object::Real(1.0)),
            ("1.9 floor", Object::Real(1.0)),
            ("1.9999999 floor", Object::Real(1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_round() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 round", Object::Integer(1)),
            ("1.0 round", Object::Real(1.0)),
            ("1.0000001 round", Object::Real(1.0)),
            ("1.1 round", Object::Real(1.0)),
            ("1.2 round", Object::Real(1.0)),
            ("1.3 round", Object::Real(1.0)),
            ("1.4 round", Object::Real(1.0)),
            ("1.4999999 round", Object::Real(1.0)),
            ("1.5 round", Object::Real(2.0)),
            ("1.6 round", Object::Real(2.0)),
            ("1.7 round", Object::Real(2.0)),
            ("1.8 round", Object::Real(2.0)),
            ("1.9 round", Object::Real(2.0)),
            ("1.9999999 round", Object::Real(2.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_truncate() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 truncate", Object::Integer(1)),
            ("1.0 truncate", Object::Real(1.0)),
            ("1.0000001 truncate", Object::Real(1.0)),
            ("1.1 truncate", Object::Real(1.0)),
            ("1.2 truncate", Object::Real(1.0)),
            ("1.3 truncate", Object::Real(1.0)),
            ("1.4 truncate", Object::Real(1.0)),
            ("1.5 truncate", Object::Real(1.0)),
            ("1.6 truncate", Object::Real(1.0)),
            ("1.7 truncate", Object::Real(1.0)),
            ("1.8 truncate", Object::Real(1.0)),
            ("1.9 truncate", Object::Real(1.0)),
            ("1.9999999 truncate", Object::Real(1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_sqrt() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 sqrt", 1.0),
            ("1.0 sqrt", 1.0),
            ("2 sqrt", 1.414213),
            ("2.0 sqrt", 1.414213),
            ("4 sqrt", 2.0),
            ("4.0 sqrt", 2.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());

            let received = interpreter.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_atan() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("0 1 atan", 0.0),
            ("0.0 1.0 atan", 0.0),
            ("1 0 atan", 90.0),
            ("1.0 0.0 atan", 90.0),
            ("-100 0 atan", 270.0),
            ("-100.0 0.0 atan", 270.0),
            ("4 4 atan", 45.0),
            ("4.0 4.0 atan", 45.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(Object::Real(expect)), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_cos() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("0 cos", 1.0),
            ("0.0 cos", 1.0),
            ("90 cos", 0.0),
            ("-90.0 cos", 0.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());

            let received = interpreter.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_sin() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("0 sin", 0.0),
            ("0.0 sin", 0.0),
            ("90 sin", 1.0),
            ("-90.0 sin", -1.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());

            let received = interpreter.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_exp() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 100 exp", 1.0),
            ("1.0 100.0 exp", 1.0),
            ("2 8 exp", 256.0),
            ("2.0 8.0 exp", 256.0),
            ("8 0.33333333 exp", 2.0),
            ("8.0 0.33333333 exp", 2.0),
            ("9 0.5 exp", 3.0),
            ("9.0 0.5 exp", 3.0),
            ("-9 -1 exp", -0.111111),
            ("-9.0 -1.0 exp", -0.111111),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());

            let received = interpreter.pop_real()?;
            assert!((received - expect) < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_ln() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("10 ln", 2.302585),
            ("10.0 ln", 2.302585),
            ("100 ln", 4.60517),
            ("100.0 ln", 4.60517),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());

            let received = interpreter.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_log() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("10 log", 1.0),
            ("10.0 log", 1.0),
            ("100 log", 2.0),
            ("100.0 log", 2.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());

            let received = interpreter.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_rand() -> Result<(), Box<dyn error::Error>> {
        for _ in 0..5 {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate("rand".chars().into())?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert!(interpreter.pop_int()? >= 0);
        }

        Ok(())
    }

    #[test]
    fn test_srand() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("100 srand rand".chars().into())?;
        let initial_value = interpreter.pop_int()?;

        interpreter.operand_stack.clear();
        interpreter.evaluate("1 srand rand".chars().into())?;
        let intermediate_value = interpreter.pop_int()?;
        assert_ne!(initial_value, intermediate_value);

        interpreter.operand_stack.clear();
        interpreter.evaluate("100 srand rand".chars().into())?;
        let end_value = interpreter.pop_int()?;
        assert_eq!(initial_value, end_value);

        Ok(())
    }

    #[test]
    fn test_rrand() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 srand rrand".chars().into())?;

        assert_eq!(1, interpreter.pop_int()?);

        Ok(())
    }
}
