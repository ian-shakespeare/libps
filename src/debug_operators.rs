use crate::{Context, Object};

pub fn assert(ctx: &mut Context) -> crate::Result<()> {
    assert!(ctx.pop_bool()?);

    Ok(())
}

pub fn asserteq(ctx: &mut Context) -> crate::Result<()> {
    let rhs = ctx.pop()?;
    let lhs = ctx.pop()?;

    assert_eq!(lhs, rhs);

    Ok(())
}

pub fn assertne(ctx: &mut Context) -> crate::Result<()> {
    let rhs = ctx.pop()?;
    let lhs = ctx.pop()?;

    assert_ne!(lhs, rhs);

    Ok(())
}

pub fn assertdeepeq(ctx: &mut Context) -> crate::Result<()> {
    let rhs = ctx.pop()?;
    let lhs = ctx.pop()?;

    match (lhs, rhs) {
        (Object::Array(lhs), Object::Array(rhs)) => {
            let lhs = ctx.get_array(lhs)?.clone();
            let rhs = ctx.get_array(rhs)?.clone();

            for (lhs, rhs) in lhs.into_iter().zip(rhs.into_iter()) {
                ctx.push(lhs);
                ctx.push(rhs);
                assertdeepeq(ctx)?;
            }
        },
        (Object::Dictionary(lhs), Object::Dictionary(rhs)) => {
            let lhs = ctx.get_dict(lhs)?.clone();
            let rhs = ctx.get_dict(rhs)?.clone();

            for (key, lhs_obj) in lhs.into_iter() {
                let rhs_obj = rhs.get(&key)?.clone();

                ctx.push(lhs_obj);
                ctx.push(rhs_obj);
                assertdeepeq(ctx)?;
            }
        },
        (Object::String(lhs), Object::String(rhs)) => {
            let lhs: &str = ctx.get_string(lhs)?.into();
            let rhs: &str = ctx.get_string(rhs)?.into();

            assert_eq!(lhs, rhs);
        },
        (lhs, rhs) => assert_eq!(lhs, rhs),
    }

    Ok(())
}
