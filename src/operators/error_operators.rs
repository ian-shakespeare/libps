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

pub fn dictstackunderflow(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::DictStackUnderflow)
}

pub fn invalidaccess(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::InvalidAccess)
}

pub fn ioerror(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::IoError)
}

pub fn limitcheck(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::LimitCheck)
}

pub fn rangecheck(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::RangeCheck)
}

pub fn stackunderflow(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::StackUnderflow)
}

pub fn syntaxerror(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::SyntaxError)
}

pub fn typecheck(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::TypeCheck)
}

pub fn undefined(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::Undefined)
}

pub fn undefinedresult(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::UndefinedResult)
}

pub fn unmatchedmark(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::UnmatchedMark)
}

pub fn unregistered(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::Unregistered)
}

pub fn vmerror(ctx: &mut Context) -> crate::Result<()> {
    recover_from_error(ctx, ErrorKind::VmError)
}
