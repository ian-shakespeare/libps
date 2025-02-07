use crate::{
    interpreter::InterpreterState,
    object::{Access, Composite},
    Error, ErrorKind, Object,
};

use super::usize_to_i32;

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

pub fn proc(state: &mut InterpreterState) -> crate::Result<()> {
    let mut inner = Vec::new();

    loop {
        let obj = state.pop()?;
        if let Object::Name(ref name) = obj {
            if name == "}" {
                break;
            }
        }

        inner.push(obj);
    }

    let composite = Composite {
        access: Access::ExecuteOnly,
        len: inner.len(),
        inner,
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
