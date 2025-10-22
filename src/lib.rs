#![feature(allocator_api)]

use std::result;

pub use error::{Error, ErrorKind};
pub use file::FileObject;
pub use object::{Mode, Object};

use crate::context::Context;

mod array;
mod context;
mod dictionary;
mod encoding;
mod error;
mod file;
mod memory;
mod name;
mod object;
mod operator;
mod scanner;
mod string;

type Result<T> = result::Result<T, Error>;

// TODO: error handling
pub fn exec(ctx: &mut Context) {
    let obj = ctx.operand_stack.pop().unwrap();

    match obj {
        Object::String(cmp) => {
            let string = ctx.get_composite_value(cmp.clone()).unwrap();
            match string.mode {
                Mode::Literal => ctx.operand_stack.push(Object::String(cmp)),
                Mode::Executable => {
                    ctx.execution_stack.push(Object::String(cmp));
                    for obj in ctx.lex().unwrap() {
                        ctx.operand_stack.push(obj);
                        exec(ctx);
                    }
                    _ = ctx.execution_stack.pop();
                },
            }
        },
        _ => todo!(),
    };
}
