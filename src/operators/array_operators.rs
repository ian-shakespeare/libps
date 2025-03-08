use crate::{
    execute_object,
    object::{Access, Mode, NameObject},
    ArrayObject, Context, Error, ErrorKind, Object, StringObject,
};

use super::usize_to_i32;

pub fn array(ctx: &mut Context) -> crate::Result<()> {
    let len = ctx.pop_usize()?;

    let arr = ArrayObject::new(vec![Object::Null; len], Access::Unlimited, Mode::Literal);
    let idx = ctx.mem_mut().insert(arr);

    ctx.push(Object::Array(idx));

    Ok(())
}

pub fn endarray(ctx: &mut Context) -> crate::Result<()> {
    let mut arr = Vec::new();

    loop {
        match ctx.pop() {
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
    let arr = ArrayObject::new(arr, Access::Unlimited, Mode::Literal);
    let idx = ctx.mem_mut().insert(arr);

    ctx.push(Object::Array(idx));

    Ok(())
}

pub fn length(ctx: &mut Context) -> crate::Result<()> {
    let obj = ctx.pop()?;

    let len = match obj {
        Object::Array(idx) | Object::Dictionary(idx) => {
            let arr = ctx.get_array(idx)?;

            if !arr.access().is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            Ok(arr.len())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    let i = usize_to_i32(len)?;
    ctx.push(Object::Integer(i));

    Ok(())
}

pub fn get(ctx: &mut Context) -> crate::Result<()> {
    let key = ctx.pop()?;
    let obj = ctx.pop()?;

    let obj = match obj {
        Object::Array(idx) => {
            let arr = ctx.get_array(idx)?;

            if arr.access().is_exec_only() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let index = key.into_usize()?;
            let obj = arr.get(index)?;

            Ok(obj.clone())
        },
        Object::Dictionary(idx) => {
            let dict = ctx.get_dict(idx)?;

            if !dict.access().is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let key = ctx.stringify(&key)?;
            let obj = dict.get(&key)?;

            Ok(obj.clone())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    ctx.push(obj);

    Ok(())
}

pub fn put(ctx: &mut Context) -> crate::Result<()> {
    let value = ctx.pop()?;
    let key = ctx.pop()?;
    let obj = ctx.pop()?;

    match obj {
        Object::Array(idx) => {
            let arr = ctx.get_array_mut(idx)?;

            if !arr.access().is_writeable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            let index = key.into_usize()?;

            let obj = arr.get_mut(index)?;
            *obj = value;

            Ok(())
        },
        Object::Dictionary(idx) => {
            let key = ctx.stringify(&key)?;
            let dict = ctx.get_dict_mut(idx)?;

            if !dict.access().is_writeable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            dict.insert(key, value);

            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }
}

pub fn getinterval(ctx: &mut Context) -> crate::Result<()> {
    let count = ctx.pop_usize()?;
    let index = ctx.pop_usize()?;
    let obj = ctx.pop()?;

    match obj {
        Object::Array(idx) => {
            let arr = ctx.get_array(idx)?;

            if !arr.access().is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            if index > arr.len() {
                return Err(Error::from(ErrorKind::RangeCheck));
            }

            let mut subarr = Vec::with_capacity(count);
            for i in index..(index + count) {
                let obj = arr.get(i)?;
                subarr.push(obj.clone());
            }

            let idx =
                ctx.mem_mut()
                    .insert(ArrayObject::new(subarr, Access::default(), Mode::default()));
            ctx.push(Object::Array(idx));

            Ok(())
        },
        Object::String(idx) => {
            let string: &str = ctx.get_string(idx)?.into();

            if string.len() <= index || string.len() <= (index + count) {
                return Err(Error::from(ErrorKind::RangeCheck));
            }

            let mut substr = String::with_capacity(count);
            for ch in string.chars().skip(index).take(count) {
                substr.push(ch);
            }

            let idx = ctx.mem_mut().insert(StringObject::from(substr));
            ctx.push(Object::String(idx));

            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }
}

pub fn putinterval(ctx: &mut Context) -> crate::Result<()> {
    let source = ctx.pop_array()?;

    if !source.access().is_readable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    let source = source.clone();

    let index = ctx.pop_usize()?;
    let destination = ctx.pop_array_mut()?;

    if !destination.access().is_writeable() {
        return Err(Error::from(ErrorKind::InvalidAccess));
    }

    for (offset, obj) in source.into_iter().enumerate() {
        let dest_obj = destination.get_mut(index + offset)?;
        *dest_obj = obj;
    }

    Ok(())
}

pub fn astore(ctx: &mut Context) -> crate::Result<()> {
    let idx = ctx.pop()?.into_index()?;
    let mut arr = ctx.get_array(idx)?.clone();

    for i in (0..arr.len()).rev() {
        let obj = arr.get_mut(i)?;
        *obj = ctx.pop()?;
    }

    let target = ctx.get_array_mut(idx)?;
    *target = arr;

    Ok(())
}

pub fn aload(ctx: &mut Context) -> crate::Result<()> {
    let obj = ctx.pop()?;

    let (idx, arr) = match obj {
        Object::Array(idx) => {
            let arr = ctx.get_array(idx)?;

            if !arr.access().is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            Ok((idx, arr.clone()))
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }?;

    for obj in arr {
        ctx.push(obj);
    }

    ctx.push(Object::Array(idx));

    Ok(())
}

pub fn forall(ctx: &mut Context) -> crate::Result<()> {
    let proc = ctx.pop()?;
    let obj = ctx.pop()?;

    match obj {
        Object::Array(idx) => {
            let arr = ctx.get_array(idx)?.clone();

            for obj in arr.into_iter() {
                ctx.push(obj);
                execute_object(ctx, proc.clone());
            }

            Ok(())
        },
        Object::Dictionary(idx) => {
            let dict = ctx.get_dict(idx)?;

            for (key, value) in dict.clone() {
                let name = NameObject::new(key, Mode::Literal);
                ctx.push(Object::Name(name));
                ctx.push(value);

                execute_object(ctx, proc.clone());
            }

            Ok(())
        },
        _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
    }
}

pub fn packedarray(ctx: &mut Context) -> crate::Result<()> {
    let len = ctx.pop_usize()?;

    let arr = ArrayObject::new(vec![Object::Null; len], Access::ReadOnly, Mode::Literal);
    let idx = ctx.mem_mut().insert(arr);

    ctx.push(Object::Array(idx));

    Ok(())
}

pub fn setpacking(ctx: &mut Context) -> crate::Result<()> {
    let b = ctx.pop_bool()?;

    ctx.is_packing = b;

    Ok(())
}

pub fn currentpacking(ctx: &mut Context) -> crate::Result<()> {
    ctx.push(Object::Boolean(ctx.is_packing));

    Ok(())
}
