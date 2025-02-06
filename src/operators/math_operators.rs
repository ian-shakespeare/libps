use crate::{interpreter::InterpreterState, Error, ErrorKind, Object};

use super::{degrees_to_radians, is_valid_real, positive_degrees, radians_to_degrees};

pub fn arithmetic(
    state: &mut InterpreterState,
    checked: impl Fn(i32, i32) -> Option<i32>,
    real: impl Fn(f64, f64) -> f64,
) -> crate::Result<()> {
    let rhs = state.pop()?;
    let lhs = state.pop()?;

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

        state.push(obj);

        return Ok(());
    }

    let total = real(lhs.into_real()?, rhs.into_real()?);

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    state.push(Object::Real(total));

    Ok(())
}

pub fn num_unary(
    state: &mut InterpreterState,
    int: impl Fn(i32) -> Option<i32>,
    real: impl Fn(f64) -> f64,
) -> crate::Result<()> {
    let n = state.pop()?;

    if n.is_int() {
        let obj = match int(n.into_int()?) {
            Some(total) => Object::Integer(total),
            None => Object::Real(real(n.into_real()?)),
        };

        state.push(obj);

        return Ok(());
    }

    state.push(Object::Real(real(n.into_real()?)));

    Ok(())
}

pub fn real_unary(state: &mut InterpreterState, unary: impl Fn(f64) -> f64) -> crate::Result<()> {
    let n = state.pop_real()?;

    let total = unary(n);

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    state.push(Object::Real(total));

    Ok(())
}

pub fn idiv(state: &mut InterpreterState) -> crate::Result<()> {
    let rhs = state.pop_int()?;
    let lhs = state.pop_int()?;

    let total = match lhs.checked_div(rhs) {
        Some(total) => Ok(total),
        None => Err(Error::from(ErrorKind::UndefinedResult)),
    }?;

    state.push(Object::Integer(total));

    Ok(())
}

pub fn imod(state: &mut InterpreterState) -> crate::Result<()> {
    let rhs = state.pop_int()?;
    let lhs = state.pop_int()?;

    let total = match lhs.checked_rem(rhs) {
        Some(total) => Ok(total),
        None => Err(Error::from(ErrorKind::UndefinedResult)),
    }?;

    state.push(Object::Integer(total));

    Ok(())
}

pub fn atan(state: &mut InterpreterState) -> crate::Result<()> {
    let den = state.pop_real()?;
    let num = state.pop_real()?;

    let total = positive_degrees(radians_to_degrees((num / den).atan()));

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    state.push(Object::Real(total));

    Ok(())
}

pub fn cos(state: &mut InterpreterState) -> crate::Result<()> {
    let n = state.pop_real()?;

    let total = degrees_to_radians(n).cos();

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    state.push(Object::Real(total));

    Ok(())
}

pub fn sin(state: &mut InterpreterState) -> crate::Result<()> {
    let n = state.pop_real()?;

    let total = degrees_to_radians(n).sin();

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    state.push(Object::Real(total));

    Ok(())
}

pub fn rand(state: &mut InterpreterState) -> crate::Result<()> {
    let n = state.rng.rand().abs();

    state.push(Object::Integer(n));

    Ok(())
}

pub fn srand(state: &mut InterpreterState) -> crate::Result<()> {
    let seed = state.pop_int()?;

    state.rng.set_seed(seed);

    Ok(())
}

pub fn rrand(state: &mut InterpreterState) -> crate::Result<()> {
    let seed = state.rng.seed();

    state.push(Object::Integer(seed));

    Ok(())
}
