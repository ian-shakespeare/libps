use crate::{interpreter::InterpreterState, Error, ErrorKind, Object};

use super::usize_to_i32;

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

            if source.access().is_exec_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let source = source.value().clone();

            let destination = state.arrays.get_mut(dest_idx)?;

            if destination.access().is_read_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            for (index, obj) in source.into_iter().enumerate() {
                match destination.value_mut().get_mut(index) {
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
