use crate::{Context, Object};

pub fn eq(ctx: &mut Context) -> crate::Result<()> {
    let rhs = ctx.pop()?;
    let lhs = ctx.pop()?;

    if lhs.is_numeric() && rhs.is_numeric() {
        let lhs = lhs.into_real()?;
        let rhs = rhs.into_real()?;

        ctx.push(Object::Boolean(lhs == rhs));
        return Ok(());
    }

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
        (Object::Name(lhs), Object::Name(rhs)) => ctx.push(Object::Boolean(lhs == rhs)),
        (Object::Array(lhs), Object::Array(rhs)) => ctx.push(Object::Boolean(lhs == rhs)),
        (Object::Dictionary(lhs), Object::Dictionary(rhs)) => ctx.push(Object::Boolean(lhs == rhs)),
        _ => ctx.push(Object::Boolean(false)),
    }

    Ok(())
}
