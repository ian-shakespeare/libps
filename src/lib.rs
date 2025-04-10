use std::io;

pub use context::Context;
pub use error::{Error, ErrorKind};
use lexer::Lexer;
use object::{Access, DictionaryObject, Mode, OperatorObject};
pub use object::{ArrayObject, Object, StringObject};

mod container;
mod context;
mod encoding;
mod error;
mod lexer;
mod object;
mod operators;
mod rand;

pub type Result<T> = std::result::Result<T, crate::Error>;

pub fn evaluate(ctx: &mut Context, input: &str) -> crate::Result<()> {
    let mut lexer = Lexer::new(input.chars());

    while let Some(obj) = lexer.lex(ctx) {
        let obj = obj?;

        if obj.is_array() && obj.mode(ctx)? == Mode::Executable {
            ctx.push(obj);
            continue;
        }

        execute_object(ctx, obj);
    }

    Ok(())
}

fn execute_object(ctx: &mut Context, obj: Object) {
    let snapshot = ctx.operand_stack.clone();

    let mode = match obj.mode(ctx) {
        Ok(mode) => mode,
        Err(e) => {
            handle_error(ctx, e, obj, snapshot).expect("failed to handle error");
            return;
        },
    };

    if mode == Mode::Literal {
        ctx.operand_stack.push(obj);
        return;
    }

    match obj {
        Object::Boolean(_)
        | Object::Integer(_)
        | Object::Real(_)
        | Object::String(_)
        | Object::Dictionary(_) => {
            ctx.operand_stack.push(obj);
        },
        Object::Array(idx) => {
            let array = match ctx.get_array(idx).cloned() {
                Ok(array) => array,
                Err(e) => {
                    handle_error(ctx, e, obj, snapshot).expect("failed to handle error");
                    return;
                },
            };

            if !array.access().is_executable() {
                handle_error(ctx, Error::from(ErrorKind::InvalidAccess), obj, snapshot)
                    .expect("failed to handle error");
                return;
            }

            for obj in array.into_iter() {
                execute_object(ctx, obj);
            }
        },
        Object::Name(ref name) => {
            let obj = match ctx.find_def(name).cloned() {
                Ok(obj) => obj,
                Err(e) => {
                    handle_error(ctx, e, obj, snapshot).expect("failed to handle error");
                    return;
                },
            };
            execute_object(ctx, obj);
        },
        Object::Operator(op) => {
            if let Err(e) = match op {
                OperatorObject::Dup => operators::dup(ctx),
                OperatorObject::Exch => operators::exch(ctx),
                OperatorObject::Pop => match ctx.pop() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
                OperatorObject::Copy => operators::copy(ctx),
                OperatorObject::Roll => operators::roll(ctx),
                OperatorObject::Index => operators::index(ctx),
                OperatorObject::Clear => operators::clear(ctx),
                OperatorObject::Count => operators::count(ctx),
                OperatorObject::Counttomark => operators::counttomark(ctx),
                OperatorObject::Cleartomark => operators::cleartomark(ctx),
                OperatorObject::Add => {
                    operators::arithmetic(ctx, i32::checked_add, |a: f64, b: f64| a + b)
                },
                OperatorObject::Div => {
                    operators::arithmetic(ctx, |_, _| None, |a: f64, b: f64| a / b)
                },
                OperatorObject::Idiv => operators::idiv(ctx),
                OperatorObject::Mod => operators::imod(ctx),
                OperatorObject::Mul => {
                    operators::arithmetic(ctx, i32::checked_mul, |a: f64, b: f64| a * b)
                },
                OperatorObject::Sub => {
                    operators::arithmetic(ctx, i32::checked_sub, |a: f64, b: f64| a - b)
                },
                OperatorObject::Abs => operators::num_unary(ctx, i32::checked_abs, f64::abs),
                OperatorObject::Neg => {
                    operators::num_unary(ctx, i32::checked_neg, |a: f64| -1.0 * a)
                },
                OperatorObject::Ceiling => operators::num_unary(ctx, |a: i32| Some(a), f64::ceil),
                OperatorObject::Floor => operators::num_unary(ctx, |a: i32| Some(a), f64::floor),
                OperatorObject::Round => operators::round(ctx),
                OperatorObject::Truncate => operators::num_unary(ctx, |a: i32| Some(a), f64::trunc),
                OperatorObject::Sqrt => operators::sqrt(ctx),
                OperatorObject::Atan => operators::atan(ctx),
                OperatorObject::Cos => operators::cos(ctx),
                OperatorObject::Sin => operators::sin(ctx),
                OperatorObject::Exp => {
                    operators::arithmetic(ctx, |_, _| None, |base: f64, exp: f64| base.powf(exp))
                },
                OperatorObject::Ln => operators::ln(ctx),
                OperatorObject::Log => operators::log(ctx),
                OperatorObject::Rand => operators::rand(ctx),
                OperatorObject::Srand => operators::srand(ctx),
                OperatorObject::Rrand => operators::rrand(ctx),
                OperatorObject::Array => operators::array(ctx),
                OperatorObject::EndArray => operators::endarray(ctx),
                OperatorObject::Length => operators::length(ctx),
                OperatorObject::Get => operators::get(ctx),
                OperatorObject::Put => operators::put(ctx),
                OperatorObject::Getinterval => operators::getinterval(ctx),
                OperatorObject::Putinterval => operators::putinterval(ctx),
                OperatorObject::Astore => operators::astore(ctx),
                OperatorObject::Aload => operators::aload(ctx),
                OperatorObject::Forall => operators::forall(ctx),
                OperatorObject::Packedarray => operators::packedarray(ctx),
                OperatorObject::Setpacking => operators::setpacking(ctx),
                OperatorObject::Currentpacking => operators::currentpacking(ctx),
                OperatorObject::Dict => operators::dict(ctx),
                OperatorObject::EndDict => operators::enddict(ctx),
                OperatorObject::Maxlength => operators::maxlength(ctx),
                OperatorObject::Begin => operators::begin(ctx),
                OperatorObject::End => operators::end(ctx),
                OperatorObject::Def => operators::def(ctx),
                OperatorObject::Load => operators::load(ctx),
                OperatorObject::Store => operators::store(ctx),
                OperatorObject::Undef => operators::undef(ctx),
                OperatorObject::Known => operators::known(ctx),
                OperatorObject::Where => operators::wheredef(ctx),
                OperatorObject::Currentdict => operators::currentdict(ctx),
                OperatorObject::Countdictstack => operators::countdictstack(ctx),
                OperatorObject::Eq => operators::eq(ctx),
                OperatorObject::Type => operators::gettype(ctx),
                OperatorObject::Handleerror => operators::handleerror(ctx),
                OperatorObject::Dictstackunderflow => {
                    operators::recover_from_error(ctx, ErrorKind::DictStackUnderflow)
                },
                OperatorObject::Invalidaccess => {
                    operators::recover_from_error(ctx, ErrorKind::InvalidAccess)
                },
                OperatorObject::Ioerror => operators::recover_from_error(ctx, ErrorKind::IoError),
                OperatorObject::Limitcheck => {
                    operators::recover_from_error(ctx, ErrorKind::LimitCheck)
                },
                OperatorObject::Rangecheck => {
                    operators::recover_from_error(ctx, ErrorKind::RangeCheck)
                },
                OperatorObject::Stackunderflow => {
                    operators::recover_from_error(ctx, ErrorKind::StackUnderflow)
                },
                OperatorObject::Syntaxerror => {
                    operators::recover_from_error(ctx, ErrorKind::SyntaxError)
                },
                OperatorObject::Typecheck => {
                    operators::recover_from_error(ctx, ErrorKind::TypeCheck)
                },
                OperatorObject::Undefined => {
                    operators::recover_from_error(ctx, ErrorKind::Undefined)
                },
                OperatorObject::Undefinedresult => {
                    operators::recover_from_error(ctx, ErrorKind::UndefinedResult)
                },
                OperatorObject::Unmatchedmark => {
                    operators::recover_from_error(ctx, ErrorKind::UnmatchedMark)
                },
                OperatorObject::Unregistered => {
                    operators::recover_from_error(ctx, ErrorKind::Unregistered)
                },
                OperatorObject::Vmerror => operators::recover_from_error(ctx, ErrorKind::VmError),
            } {
                handle_error(ctx, e, Object::Operator(op), snapshot)
                    .expect("failed to handle error");
            }
        },
        _ => {
            handle_error(
                ctx,
                Error::new(ErrorKind::Unregistered, "not implemented"),
                obj,
                snapshot,
            )
            .expect("failed to handle error");
        },
    }
}

pub fn handle_error(
    ctx: &mut Context,
    e: Error,
    cause: Object,
    stack_snapshot: Vec<Object>,
) -> crate::Result<()> {
    // Recover the operand stack
    ctx.operand_stack = stack_snapshot;
    ctx.push(cause);

    // Execute error handler
    let idx = ctx.find_def("errordict").cloned()?.into_index()?;
    let error_dict = ctx.get_dict(idx)?;
    let handler = error_dict.get(e.kind().into()).cloned()?;
    execute_object(ctx, handler);

    Ok(())
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
