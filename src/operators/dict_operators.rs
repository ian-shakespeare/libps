use std::collections::HashMap;

use crate::{object::DictionaryObject, Context, Error, ErrorKind, Object};

use super::{i32_to_usize, usize_to_i32};

pub fn dict(ctx: &mut Context) -> crate::Result<()> {
    // use `i32_to_usize` to send `limitcheck` errors
    let capacity = i32_to_usize(ctx.pop_int()?)?;

    let idx = ctx
        .mem_mut()
        .insert(DictionaryObject::with_capacity(capacity));

    ctx.push(Object::Dictionary(idx));

    Ok(())
}

pub fn enddict(ctx: &mut Context) -> crate::Result<()> {
    let mut dict = HashMap::new();

    loop {
        let value = ctx.pop()?;
        if value.is_mark() {
            break;
        }

        let key = ctx.pop()?;
        if key.is_mark() {
            return Err(Error::from(ErrorKind::RangeCheck));
        }

        let key = ctx.stringify(&key)?;

        dict.insert(key, value);
    }

    let idx = ctx.mem_mut().insert(DictionaryObject::from(dict));
    ctx.push(Object::Dictionary(idx));

    Ok(())
}

pub fn maxlength(ctx: &mut Context) -> crate::Result<()> {
    let dict = ctx.pop_dict()?;

    if !dict.access().is_readable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    let capacity = dict.capacity();

    let capacity = usize_to_i32(capacity)?;

    ctx.push(Object::Integer(capacity));

    Ok(())
}

pub fn begin(ctx: &mut Context) -> crate::Result<()> {
    let obj = ctx.pop()?;

    match obj {
        Object::Dictionary(idx) => {
            ctx.dict_stack.push(idx);
            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
    }
}

pub fn end(ctx: &mut Context) -> crate::Result<()> {
    if ctx.dict_stack.len() <= 3 {
        return Err(Error::from(ErrorKind::DictStackUnderflow));
    }

    let _ = ctx.dict_stack.pop();

    Ok(())
}

pub fn def(ctx: &mut Context) -> crate::Result<()> {
    let value = ctx.pop()?;
    let key_obj = ctx.pop()?;
    let key = ctx.stringify(&key_obj)?;

    let dict_idx = ctx
        .dict_stack
        .last()
        .ok_or(Error::from(ErrorKind::LimitCheck))?;

    let dict = ctx.get_dict_mut(*dict_idx)?;

    if !dict.access().is_writeable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    dict.insert(key, value);

    Ok(())
}

pub fn load(ctx: &mut Context) -> crate::Result<()> {
    let key_obj = ctx.pop()?;
    let key = ctx.stringify(&key_obj)?;

    let obj = ctx.find_def(key)?;

    ctx.push(obj.clone());

    Ok(())
}

pub fn store(ctx: &mut Context) -> crate::Result<()> {
    let value = ctx.pop()?;
    let key_obj = ctx.pop()?;
    let key = ctx.stringify(&key_obj)?;

    let dict = match ctx.find_dict_mut(&key) {
        Ok(dict) => dict,
        Err(e) => {
            if e.kind() != ErrorKind::Undefined {
                return Err(e);
            }

            ctx.get_dict_mut(
                *ctx.dict_stack
                    .last()
                    .ok_or(Error::from(ErrorKind::DictStackUnderflow))?,
            )?
        },
    };

    dict.insert(key, value);

    Ok(())
}

pub fn undef(ctx: &mut Context) -> crate::Result<()> {
    let key_obj = ctx.pop()?;
    let key = ctx.stringify(&key_obj)?;
    let dict = ctx.pop_dict_mut()?;

    if !dict.access().is_writeable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    match dict.remove(&key) {
        Some(..) => Ok(()),
        None => Err(Error::new(ErrorKind::Undefined, key)),
    }
}

pub fn known(ctx: &mut Context) -> crate::Result<()> {
    let key_obj = ctx.pop()?;
    let key = ctx.stringify(&key_obj)?;
    let dict = ctx.pop_dict()?;

    if !dict.access().is_readable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    let exists = dict.contains_key(&key);

    ctx.push(Object::Boolean(exists));

    Ok(())
}

pub fn wheredef(ctx: &mut Context) -> crate::Result<()> {
    let key_obj = ctx.pop()?;
    let key = ctx.stringify(&key_obj)?;

    match ctx.find_index(key) {
        Ok(idx) => {
            ctx.push(Object::Dictionary(idx));
            ctx.push(Object::Boolean(true));
        },
        Err(..) => {
            ctx.push(Object::Boolean(false));
        },
    }

    Ok(())
}

pub fn currentdict(ctx: &mut Context) -> crate::Result<()> {
    let idx = ctx
        .dict_stack
        .last()
        .ok_or(Error::from(ErrorKind::DictStackUnderflow))?;

    ctx.push(Object::Dictionary(*idx));

    Ok(())
}

pub fn errorstatus(_ctx: &mut Context) -> crate::Result<()> {
    Err(Error::new(ErrorKind::Unregistered, "not implemented"))
}

pub fn userdict(_ctx: &mut Context) -> crate::Result<()> {
    Err(Error::new(ErrorKind::Unregistered, "not implemented"))
}

pub fn globaldict(_ctx: &mut Context) -> crate::Result<()> {
    Err(Error::new(ErrorKind::Unregistered, "not implemented"))
}

pub fn statusdict(_ctx: &mut Context) -> crate::Result<()> {
    Err(Error::new(ErrorKind::Unregistered, "not implemented"))
}

pub fn countdictstack(ctx: &mut Context) -> crate::Result<()> {
    let count = usize_to_i32(ctx.dict_stack.len())?;

    ctx.push(Object::Integer(count));

    Ok(())
}

pub fn dictstack(_ctx: &mut Context) -> crate::Result<()> {
    Err(Error::new(ErrorKind::Unregistered, "not implemented"))
}

pub fn cleardictstack(_ctx: &mut Context) -> crate::Result<()> {
    Err(Error::new(ErrorKind::Unregistered, "not implemented"))
}
