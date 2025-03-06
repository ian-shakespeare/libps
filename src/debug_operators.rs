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
            let arr1 = ctx.get_array(lhs)?.clone();
            let arr2 = ctx.get_array(rhs)?.clone();

            for (lhs, rhs) in arr1.into_iter().zip(arr2.into_iter()) {
                ctx.push(lhs);
                ctx.push(rhs);
                assertdeepeq(ctx)?;
            }
        },
        (Object::Dictionary(lhs), Object::Dictionary(rhs)) => {
            let dict1 = ctx.get_dict(lhs)?.clone();
            let dict2 = ctx.get_dict(rhs)?.clone();

            for ((left_key, left_obj), (right_key, right_obj)) in
                dict1.into_iter().zip(dict2.into_iter())
            {
                assert_eq!(left_key, right_key);

                ctx.push(left_obj);
                ctx.push(right_obj);
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
