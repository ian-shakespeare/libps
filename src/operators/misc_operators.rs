use crate::{Context, Object};

pub fn null(ctx: &mut Context) -> crate::Result<()> {
    ctx.push(Object::Null);

    Ok(())
}
