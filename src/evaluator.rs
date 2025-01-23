use crate::{
    execution::{ExecutionState, Procedure},
    object::Object,
    Error, ErrorKind,
};

#[derive(Default)]
pub struct Evaluator {
    execution_state: ExecutionState,
}

impl Evaluator {
    pub fn evaluate<I>(&mut self, objects: I) -> crate::Result<()>
    where
        I: Iterator<Item = Object>,
    {
        for obj in objects {
            self.evaluate_object(obj)?;
        }

        Ok(())
    }

    fn evaluate_object(&mut self, obj: Object) -> crate::Result<()> {
        match obj {
            Object::Integer(_) | Object::Real(_) | Object::Boolean(_) | Object::LiteralName(_) => {
                self.execution_state.operand_stack.push(obj);
                Ok(())
            }
            Object::Name(ref name) => {
                match self
                    .execution_state
                    .dict_stack
                    .search(|dict| dict.contains_key(name))
                {
                    None => Err(Error::from(ErrorKind::Undefined)),
                    Some(dict) => match dict.get(name) {
                        None => Err(Error::from(ErrorKind::Undefined)),
                        Some(defined) => match defined {
                            Procedure::Defined(obj) => self.evaluate_object(obj.clone()),
                            Procedure::System(op) => op(&mut self.execution_state),
                        },
                    },
                }
            }
            _ => todo!(),
        }
    }
}
