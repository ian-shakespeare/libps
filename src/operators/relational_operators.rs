use crate::{Context, Object};

pub fn eq(ctx: &mut Context) -> crate::Result<()> {
    let rhs = ctx.pop()?;
    let lhs = ctx.pop()?;

    match (lhs, rhs) {
        (Object::String(lhs), Object::String(rhs)) => {
            let lhs = ctx.get_string(lhs)?;
            let rhs = ctx.get_string(rhs)?;

            ctx.push(Object::Boolean(lhs == rhs));
        },
        (Object::String(lhs), Object::Name(rhs)) => {
            let lhs: &str = ctx.get_string(lhs)?.into();
            let rhs: &str = (&rhs).into();

            ctx.push(Object::Boolean(lhs == rhs))
        },
        (Object::Name(lhs), Object::String(rhs)) => {
            let lhs: &str = (&lhs).into();
            let rhs: &str = ctx.get_string(rhs)?.into();

            ctx.push(Object::Boolean(lhs == rhs))
        },
        (lhs, rhs) => ctx.push(Object::Boolean(lhs == rhs)),
    }

    Ok(())
}
