use crate::{Context, Error, ErrorKind, Object};

use super::usize_to_i32;

pub fn dup(ctx: &mut Context) -> crate::Result<()> {
    let obj = ctx.pop()?;

    ctx.push(obj.clone());
    ctx.push(obj);

    Ok(())
}

pub fn exch(ctx: &mut Context) -> crate::Result<()> {
    let first = ctx.pop()?;
    let second = ctx.pop()?;

    ctx.push(first);
    ctx.push(second);

    Ok(())
}

pub fn copy(ctx: &mut Context) -> crate::Result<()> {
    let mut original_stack = Vec::new();
    let mut copy_stack = Vec::new();

    match ctx.pop()? {
        Object::Integer(n) => {
            for _ in 0..n {
                let obj = ctx.pop()?;
                copy_stack.push(obj.clone());
                original_stack.push(obj);
            }

            while let Some(obj) = original_stack.pop() {
                ctx.push(obj);
            }

            while let Some(obj) = copy_stack.pop() {
                ctx.push(obj);
            }

            Ok(())
        },
        Object::Array(dest_idx) => {
            let obj = ctx.pop()?;

            let source = match obj {
                Object::Array(idx) => {
                    let arr = ctx.get_array(idx)?.clone();

                    Ok(arr)
                },
                _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
            }?;
            if source.access().is_exec_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let destination = ctx.get_array_mut(dest_idx)?;
            if !destination.access().is_writeable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            for (index, obj) in source.into_iter().enumerate() {
                let dest_obj = destination.get_mut(index)?;
                *dest_obj = obj;
            }

            ctx.push(Object::Array(dest_idx));

            Ok(())
        },
        Object::Dictionary(idx) => {
            let source = ctx.pop_dict()?.clone();
            let destination = ctx.get_dict_mut(idx)?;

            for (key, value) in source {
                destination.insert(key, value);
            }

            ctx.push(Object::Dictionary(idx));

            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
    }
}

pub fn roll(ctx: &mut Context) -> crate::Result<()> {
    let mut top_stack = Vec::new();
    let mut bottom_stack = Vec::new();

    let j = ctx.pop_int()?;
    let n = ctx.pop_int()?;

    let j = (n + j) % n;
    for i in 0..n {
        let obj = ctx.pop()?;
        if i < j {
            bottom_stack.push(obj);
        } else {
            top_stack.push(obj);
        }
    }

    while let Some(obj) = bottom_stack.pop() {
        ctx.push(obj);
    }

    while let Some(obj) = top_stack.pop() {
        ctx.push(obj);
    }

    Ok(())
}

pub fn index(ctx: &mut Context) -> crate::Result<()> {
    let idx = ctx.pop_usize()?;

    if ctx.operand_stack.is_empty() {
        return Err(Error::from(ErrorKind::RangeCheck));
    }

    let Some(idx) = (ctx.operand_stack.len() - 1).checked_sub(idx) else {
        return Err(Error::from(ErrorKind::RangeCheck));
    };

    match ctx.operand_stack.get(idx) {
        Some(obj) => {
            ctx.push(obj.clone());
            Ok(())
        },
        None => Err(Error::from(ErrorKind::RangeCheck)),
    }
}

pub fn mark(ctx: &mut Context) -> crate::Result<()> {
    ctx.push(Object::Mark);

    Ok(())
}

pub fn clear(ctx: &mut Context) -> crate::Result<()> {
    ctx.operand_stack.clear();

    Ok(())
}

pub fn count(ctx: &mut Context) -> crate::Result<()> {
    let i = usize_to_i32(ctx.operand_stack.len())?;

    ctx.push(Object::Integer(i));

    Ok(())
}

pub fn counttomark(ctx: &mut Context) -> crate::Result<()> {
    let Some((u, _)) = ctx
        .operand_stack
        .iter()
        .rev()
        .enumerate()
        .find(|(_, obj)| matches!(obj, Object::Mark))
    else {
        return Err(Error::from(ErrorKind::UnmatchedMark));
    };

    let i = usize_to_i32(u)?;

    ctx.push(Object::Integer(i));

    Ok(())
}

pub fn cleartomark(ctx: &mut Context) -> crate::Result<()> {
    loop {
        match ctx.pop() {
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
