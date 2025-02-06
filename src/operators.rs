use std::f64::consts;

use crate::{
    interpreter::InterpreterState,
    object::{Access, Composite},
    Error, ErrorKind, Object,
};

pub fn dup(state: &mut InterpreterState) -> crate::Result<()> {
    let obj = state.pop()?;

    state.push(obj.clone());
    state.push(obj);

    Ok(())
}

pub fn exch(state: &mut InterpreterState) -> crate::Result<()> {
    let first = state.pop()?;
    let second = state.pop()?;

    state.push(first);
    state.push(second);

    Ok(())
}

pub fn copy(state: &mut InterpreterState) -> crate::Result<()> {
    let mut original_stack = Vec::new();
    let mut copy_stack = Vec::new();

    match state.pop()? {
        Object::Integer(n) => {
            for _ in 0..n {
                let obj = state.pop()?;
                copy_stack.push(obj.clone());
                original_stack.push(obj);
            }

            while let Some(obj) = original_stack.pop() {
                state.push(obj);
            }

            while let Some(obj) = copy_stack.pop() {
                state.push(obj);
            }

            Ok(())
        },
        Object::Array(dest_idx) => {
            let obj = state.pop()?;

            let source = match obj {
                Object::Array(idx) | Object::PackedArray(idx) => {
                    let arr = state.arrays.get(idx)?;

                    Ok(arr)
                },
                _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
            }?;

            if source.is_exec_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let source = source.inner.clone();

            let destination = state.arrays.get_mut(dest_idx)?;

            if destination.is_read_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            for (index, obj) in source.into_iter().enumerate() {
                match destination.inner.get_mut(index) {
                    Some(dest_obj) => *dest_obj = obj,
                    None => return Err(Error::from(ErrorKind::RangeCheck)),
                }
            }

            state.push(Object::Array(dest_idx));

            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
    }
}

pub fn roll(state: &mut InterpreterState) -> crate::Result<()> {
    let mut top_stack = Vec::new();
    let mut bottom_stack = Vec::new();

    let j = state.pop_int()?;
    let n = state.pop_int()?;

    let j = (n + j) % n;
    for i in 0..n {
        let obj = state.pop()?;
        if i < j {
            bottom_stack.push(obj);
        } else {
            top_stack.push(obj);
        }
    }

    while let Some(obj) = bottom_stack.pop() {
        state.push(obj);
    }

    while let Some(obj) = top_stack.pop() {
        state.push(obj);
    }

    Ok(())
}

pub fn index(state: &mut InterpreterState) -> crate::Result<()> {
    let idx = state.pop_usize()?;

    if state.operand_stack.is_empty() {
        return Err(Error::from(ErrorKind::RangeCheck));
    }

    let Some(idx) = (state.operand_stack.len() - 1).checked_sub(idx) else {
        return Err(Error::from(ErrorKind::RangeCheck));
    };

    match state.operand_stack.get(idx) {
        Some(obj) => {
            state.push(obj.clone());
            Ok(())
        },
        None => Err(Error::from(ErrorKind::RangeCheck)),
    }
}

pub fn mark(state: &mut InterpreterState) -> crate::Result<()> {
    state.push(Object::Mark);

    Ok(())
}

pub fn clear(state: &mut InterpreterState) -> crate::Result<()> {
    state.operand_stack.clear();

    Ok(())
}

pub fn count(state: &mut InterpreterState) -> crate::Result<()> {
    let i = usize_to_i32(state.operand_stack.len())?;

    state.push(Object::Integer(i));

    Ok(())
}

pub fn counttomark(state: &mut InterpreterState) -> crate::Result<()> {
    let Some((u, _)) = state
        .operand_stack
        .iter()
        .rev()
        .enumerate()
        .find(|(_, obj)| matches!(obj, Object::Mark))
    else {
        return Err(Error::from(ErrorKind::UnmatchedMark));
    };

    let i = usize_to_i32(u)?;

    state.push(Object::Integer(i));

    Ok(())
}

pub fn cleartomark(state: &mut InterpreterState) -> crate::Result<()> {
    loop {
        match state.pop() {
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

pub fn array(state: &mut InterpreterState) -> crate::Result<()> {
    let len = state.pop_usize()?;

    let inner = vec![Object::Null; len];
    let composite = Composite {
        access: Access::default(),
        len,
        inner,
    };

    let idx = state.arrays.insert(composite);
    state.push(Object::Array(idx));

    Ok(())
}

pub fn endarray(state: &mut InterpreterState) -> crate::Result<()> {
    let mut arr = Vec::new();

    loop {
        match state.pop() {
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

    let len = arr.len();
    let composite = Composite {
        access: Access::default(),
        inner: arr,
        len,
    };

    let idx = state.arrays.insert(composite);
    state.push(Object::Array(idx));

    Ok(())
}

pub fn length(state: &mut InterpreterState) -> crate::Result<()> {
    let obj = state.pop()?;

    let len = match obj {
        Object::Array(idx) | Object::PackedArray(idx) => {
            let arr = state.arrays.get(idx)?;

            if !arr.is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            Ok(arr.len)
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    let i = usize_to_i32(len)?;
    state.push(Object::Integer(i));

    Ok(())
}

pub fn get(state: &mut InterpreterState) -> crate::Result<()> {
    let index = state.pop_usize()?;
    let obj = state.pop()?;

    let obj = match obj {
        Object::Array(idx) | Object::PackedArray(idx) => {
            let arr = state.arrays.get(idx)?;

            if arr.is_exec_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            match arr.inner.get(index) {
                Some(obj) => Ok(obj.clone()),
                None => Err(Error::from(ErrorKind::RangeCheck)),
            }
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    state.push(obj);

    Ok(())
}

pub fn put(state: &mut InterpreterState) -> crate::Result<()> {
    let value = state.pop()?;
    let index = state.pop_usize()?;
    let arr = state.pop_array_mut()?;

    if !arr.is_writeable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    let Some(obj) = arr.inner.get_mut(index) else {
        return Err(Error::from(ErrorKind::RangeCheck));
    };

    *obj = value;

    Ok(())
}

pub fn getinterval(state: &mut InterpreterState) -> crate::Result<()> {
    let count = state.pop_usize()?;
    let index = state.pop_usize()?;
    let obj = state.pop()?;

    let arr = match obj {
        Object::Array(idx) | Object::PackedArray(idx) => {
            let arr = state.arrays.get(idx)?;

            if !arr.is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            Ok(arr.inner.clone())
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

    let composite = Composite {
        access: Access::default(),
        inner: subarr,
        len: count,
    };

    let idx = state.arrays.insert(composite);
    state.push(Object::Array(idx));

    Ok(())
}

pub fn putinterval(state: &mut InterpreterState) -> crate::Result<()> {
    let source = state.pop_array()?;

    if !source.is_readable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    let source = source.inner.clone();

    let index = state.pop_usize()?;
    let destination = state.pop_array_mut()?;

    if !destination.is_writeable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    for (offset, obj) in source.into_iter().enumerate() {
        let Some(dest_obj) = destination.inner.get_mut(index + offset) else {
            return Err(Error::from(ErrorKind::RangeCheck));
        };

        *dest_obj = obj;
    }

    Ok(())
}

pub fn astore(state: &mut InterpreterState) -> crate::Result<()> {
    let Object::Array(arr_idx) = state.pop()? else {
        return Err(Error::new(ErrorKind::TypeCheck, "expected array"));
    };

    let len = match state.arrays.get(arr_idx) {
        Ok(composite) => {
            if !composite.is_writeable() {
                Err(Error::from(ErrorKind::InvalidAccess))
            } else {
                Ok(composite.len)
            }
        },
        Err(_) => Err(Error::from(ErrorKind::Undefined)),
    }?;

    let mut stored = vec![Object::Null; len];

    for i in (0..len).rev() {
        stored[i] = state.pop()?;
    }

    match state.arrays.get_mut(arr_idx) {
        Ok(composite) => {
            composite.inner = stored;
            Ok(())
        },
        Err(_) => Err(Error::from(ErrorKind::Undefined)),
    }
}

pub fn aload(state: &mut InterpreterState) -> crate::Result<()> {
    let obj = state.pop()?;

    let (idx, arr) = match obj {
        Object::Array(idx) | Object::PackedArray(idx) => {
            let arr = state.arrays.get(idx)?;

            if !arr.is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            Ok((idx, arr.inner.clone()))
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    for obj in arr {
        state.push(obj);
    }

    state.push(Object::Array(idx));

    Ok(())
}

pub fn packedarray(state: &mut InterpreterState) -> crate::Result<()> {
    let len = state.pop_usize()?;

    let inner = vec![Object::Null; len];

    let composite = Composite {
        access: Access::ReadOnly,
        len,
        inner,
    };

    let index = state.arrays.insert(composite);

    state.push(Object::PackedArray(index));

    Ok(())
}

pub fn setpacking(state: &mut InterpreterState) -> crate::Result<()> {
    let b = state.pop_bool()?;

    state.is_packing = b;

    Ok(())
}

pub fn currentpacking(state: &mut InterpreterState) -> crate::Result<()> {
    state.push(Object::Boolean(state.is_packing));

    Ok(())
}

// Utils
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

fn usize_to_i32(u: usize) -> crate::Result<i32> {
    let i: i32 = match u.try_into() {
        Ok(i) => Ok(i),
        Err(_) => Err(Error::new(
            ErrorKind::Unregistered,
            "failed to convert usize to int",
        )),
    }?;

    Ok(i)
}

fn is_valid_real(n: f64) -> bool {
    n.is_finite() && !n.is_nan()
}
