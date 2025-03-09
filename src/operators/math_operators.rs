use crate::{Context, Error, ErrorKind, Object};

use super::{degrees_to_radians, is_valid_real, positive_degrees, radians_to_degrees};

pub fn arithmetic(
    ctx: &mut Context,
    checked: impl Fn(i32, i32) -> Option<i32>,
    real: impl Fn(f64, f64) -> f64,
) -> crate::Result<()> {
    let rhs = ctx.pop()?;
    let lhs = ctx.pop()?;

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

        ctx.push(obj);

        return Ok(());
    }

    let total = real(lhs.into_real()?, rhs.into_real()?);

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    ctx.push(Object::Real(total));

    Ok(())
}

pub fn num_unary(
    ctx: &mut Context,
    int: impl Fn(i32) -> Option<i32>,
    real: impl Fn(f64) -> f64,
) -> crate::Result<()> {
    let n = ctx.pop()?;

    if n.is_int() {
        let obj = match int(n.clone().into_int()?) {
            Some(total) => Object::Integer(total),
            None => Object::Real(real(n.into_real()?)),
        };

        ctx.push(obj);

        return Ok(());
    }

    ctx.push(Object::Real(real(n.into_real()?)));

    Ok(())
}

pub fn idiv(ctx: &mut Context) -> crate::Result<()> {
    let rhs = ctx.pop_int()?;
    let lhs = ctx.pop_int()?;

    let total = match lhs.checked_div(rhs) {
        Some(total) => Ok(total),
        None => Err(Error::from(ErrorKind::UndefinedResult)),
    }?;

    ctx.push(Object::Integer(total));

    Ok(())
}

pub fn imod(ctx: &mut Context) -> crate::Result<()> {
    let rhs = ctx.pop_int()?;
    let lhs = ctx.pop_int()?;

    let total = match lhs.checked_rem(rhs) {
        Some(total) => Ok(total),
        None => Err(Error::from(ErrorKind::UndefinedResult)),
    }?;

    ctx.push(Object::Integer(total));

    Ok(())
}

pub fn round(ctx: &mut Context) -> crate::Result<()> {
    let num = ctx.pop()?;

    if num.is_int() {
        ctx.push(num);
        return Ok(());
    }

    let num = num.into_real()?;

    let total = if num < 0.0 {
        if num.fract() < -0.5 {
            num.floor()
        } else {
            num.ceil()
        }
    } else if num.fract() >= 0.5 {
        num.ceil()
    } else {
        num.floor()
    };

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    ctx.push(Object::Real(total));

    Ok(())
}

pub fn sqrt(ctx: &mut Context) -> crate::Result<()> {
    let num = ctx.pop_real()?;

    if num < 0.0 {
        return Err(Error::from(ErrorKind::RangeCheck));
    }

    let total = num.sqrt();
    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    ctx.push(Object::Real(total));

    Ok(())
}

pub fn atan(ctx: &mut Context) -> crate::Result<()> {
    let den = ctx.pop_real()?;
    let num = ctx.pop_real()?;

    let total = positive_degrees(radians_to_degrees((num / den).atan()));

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    ctx.push(Object::Real(total));

    Ok(())
}

pub fn cos(ctx: &mut Context) -> crate::Result<()> {
    let n = ctx.pop_real()?;

    let total = degrees_to_radians(n).cos();

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    ctx.push(Object::Real(total));

    Ok(())
}

pub fn sin(ctx: &mut Context) -> crate::Result<()> {
    let n = ctx.pop_real()?;

    let total = degrees_to_radians(n).sin();

    if !is_valid_real(total) {
        return Err(Error::from(ErrorKind::UndefinedResult));
    }

    ctx.push(Object::Real(total));

    Ok(())
}

pub fn ln(ctx: &mut Context) -> crate::Result<()> {
    let n = ctx.pop_real()?;

    if n < 0.0 {
        return Err(Error::from(ErrorKind::RangeCheck));
    }

    let total = n.ln();
    ctx.push(Object::Real(total));

    Ok(())
}

pub fn log(ctx: &mut Context) -> crate::Result<()> {
    let n = ctx.pop_real()?;

    if n < 0.0 {
        return Err(Error::from(ErrorKind::RangeCheck));
    }

    let total = n.log10();
    ctx.push(Object::Real(total));

    Ok(())
}

pub fn rand(ctx: &mut Context) -> crate::Result<()> {
    let n = ctx.rng.rand().abs();

    ctx.push(Object::Integer(n));

    Ok(())
}

pub fn srand(ctx: &mut Context) -> crate::Result<()> {
    let seed = ctx.pop_int()?;

    ctx.rng.set_seed(seed);

    Ok(())
}

pub fn rrand(ctx: &mut Context) -> crate::Result<()> {
    let seed = ctx.rng.seed();

    ctx.push(Object::Integer(seed));

    Ok(())
}
