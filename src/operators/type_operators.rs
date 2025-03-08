use crate::{
    object::{Mode, NameObject},
    Context, Object,
};

pub fn gettype(ctx: &mut Context) -> crate::Result<()> {
    let obj = ctx.pop()?;

    let obj_type: &str = obj.into();
    let name = NameObject::new(obj_type, Mode::Executable);

    ctx.push(Object::Name(name));

    Ok(())
}
