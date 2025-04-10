use crate::{
    object::{Access, Mode},
    ArrayObject, Context, ErrorKind, Object,
};

pub fn handleerror(ctx: &mut Context) -> crate::Result<()> {
    let idx = ctx.find_def("$error").cloned()?.into_index()?;
    let error_info = ctx.get_dict_mut(idx)?;
    error_info.insert("newerror", Object::Boolean(false));

    Ok(())
}

pub fn recover_from_error(ctx: &mut Context, e: ErrorKind) -> crate::Result<()> {
    let cause = ctx.pop()?;
    let error_name: &str = e.into();
    let ostack = ArrayObject::new(
        ctx.operand_stack.clone(),
        Access::default(),
        Mode::default(),
    );
    let ostack_idx = ctx.mem_mut().insert(ostack);

    let idx = ctx.find_def("$error").cloned()?.into_index()?;
    let error_info = ctx.get_dict_mut(idx)?;
    error_info.insert("newerror", Object::Boolean(true));
    error_info.insert("errorname", Object::Name(error_name.into()));
    error_info.insert("command", cause);
    error_info.insert("ostack", Object::Array(ostack_idx));

    Ok(())
}
