use std::io;

pub use context::Context;
pub use error::{Error, ErrorKind};
use lexer::Lexer;
use object::{Access, ArrayObject, DictionaryObject, Mode, Object, StringObject};

mod container;
mod context;
mod encoding;
mod error;
mod lexer;
mod object;
mod rand;

pub type Result<T> = std::result::Result<T, crate::Error>;

pub fn evaluate(ctx: &mut Context, input: &str) -> crate::Result<()> {
    let mut lexer = Lexer::new(input.chars());

    while let Some(obj) = lexer.lex(ctx) {
        let obj = obj?;

        if let Some(Mode::Literal) = obj.mode(ctx) {
            ctx.operand_stack.push(obj);
            continue;
        }

        execute_object(ctx, obj)?;
    }

    Ok(())
}

fn execute_object(ctx: &mut Context, obj: Object) -> crate::Result<()> {
    match obj {
        Object::Boolean(_) | Object::Integer(_) | Object::Real(_) | Object::String(_) => {
            ctx.operand_stack.push(obj);

            Ok(())
        },
        Object::Array(idx) => {
            let array: ArrayObject = ctx
                .mem()
                .get(idx)
                .ok_or(Error::from(ErrorKind::VmError))?
                .clone()
                .try_into()?;

            if !array.access().is_executable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            for obj in array.into_iter() {
                execute_object(ctx, obj)?;
            }

            Ok(())
        },
        Object::Name(name) => execute_object(ctx, ctx.find_def(&name)?.clone()),
        Object::Operator(operator) => operator(ctx),
        _ => Err(Error::new(ErrorKind::Unregistered, "not implemented")),
    }
}

pub fn write_stack(writer: &mut impl io::Write, ctx: &Context) -> io::Result<usize> {
    let mut count = 0;

    for obj in &ctx.operand_stack {
        count += writer.write(b" ")?;
        count += write_object(writer, ctx, obj)?;
    }

    Ok(count)
}

fn write_object(writer: &mut impl io::Write, ctx: &Context, obj: &Object) -> io::Result<usize> {
    match obj {
        Object::Integer(i) => writer.write(i.to_string().as_bytes()),
        Object::Real(r) => {
            let is_whole_number = r.fract() == 0.0;
            if is_whole_number {
                writer.write(format!("{:.1}", r).as_bytes())
            } else {
                writer.write(r.to_string().as_bytes())
            }
        },
        Object::Boolean(b) => writer.write(b.to_string().as_bytes()),
        Object::String(idx) => {
            let string: &StringObject = ctx
                .mem()
                .get(*idx)
                .ok_or(io::Error::from(io::ErrorKind::NotFound))?
                .try_into()
                .or(Err(io::Error::from(io::ErrorKind::InvalidData)))?;
            let string: &str = string.into();

            let output = format!("({})", string);

            writer.write(output.as_bytes())
        },
        Object::Array(idx) => {
            let array: &ArrayObject = ctx
                .mem()
                .get(*idx)
                .ok_or(io::Error::from(io::ErrorKind::NotFound))?
                .try_into()
                .or(Err(io::Error::from(io::ErrorKind::InvalidData)))?;

            let (left, right) = if array.access() == Access::ExecuteOnly {
                (b"{", b" }")
            } else {
                (b"[", b" ]")
            };
            let mut count = writer.write(left)?;

            for obj in array.clone().into_iter() {
                count += writer.write(b" ")?;
                count += write_object(writer, ctx, &obj)?;
            }

            count += writer.write(right)?;

            Ok(count)
        },
        Object::Dictionary(idx) => {
            let mut count = writer.write(b"<<")?;

            let dict: &DictionaryObject = ctx
                .mem()
                .get(*idx)
                .ok_or(io::Error::from(io::ErrorKind::NotFound))?
                .try_into()
                .or(Err(io::Error::from(io::ErrorKind::InvalidData)))?;

            for (key, value) in dict.clone().into_iter() {
                count += writer.write(b" ")?;

                let key: Vec<u8> = key.bytes().collect();
                count += writer.write(&key)?;

                count += writer.write(b" ")?;
                count += write_object(writer, ctx, &value)?;
            }

            count += writer.write(b" >>")?;

            Ok(count)
        },
        Object::Name(name) => {
            let string: &str = name.into();
            writer.write(string.as_bytes())
        },
        Object::Mark => writer.write(b"mark"),
        Object::Null => writer.write(b"null"),
        _ => Ok(0),
    }
}
