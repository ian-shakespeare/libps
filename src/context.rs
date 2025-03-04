use std::collections::HashMap;

use crate::{
    container::Container,
    object::{Access, Composite, DictionaryObject, Mode, Object},
    rand::RandomNumberGenerator,
    stack_operator::*,
    ArrayObject, Error, ErrorKind,
};

pub struct Context {
    rng: RandomNumberGenerator,
    pub operand_stack: Vec<Object>,
    local_mem: Container<Composite>,
    global_mem: Container<Composite>,
    dict_stack: Vec<usize>,
}

impl Default for Context {
    fn default() -> Self {
        let mut local_mem: Container<Composite> = Container::default();
        let system_dict = local_mem.insert(system_dict());

        Self {
            rng: RandomNumberGenerator::default(),
            operand_stack: Vec::default(),
            global_mem: Container::default(),
            dict_stack: vec![system_dict],
            local_mem,
        }
    }
}

impl<'a> Context {
    pub fn find_def<S>(&'a self, key: S) -> crate::Result<&'a Object>
    where
        S: Into<&'a str>,
    {
        let key: &str = key.into();

        for idx in self.dict_stack.iter().rev() {
            if let Some(comp) = self.mem().get(*idx) {
                let dict: &DictionaryObject = comp.try_into()?;

                if let Some(obj) = dict.get(key) {
                    return Ok(obj);
                }
            }
        }

        Err(Error::new(ErrorKind::Undefined, key))
    }

    pub fn get_array(&self, index: usize) -> crate::Result<&ArrayObject> {
        let arr: &ArrayObject = self
            .mem()
            .get(index)
            .ok_or(Error::from(ErrorKind::VmError))?
            .try_into()?;

        Ok(arr)
    }

    pub fn get_array_mut(&mut self, index: usize) -> crate::Result<&mut ArrayObject> {
        let arr: &mut ArrayObject = self
            .mem_mut()
            .get_mut(index)
            .ok_or(Error::from(ErrorKind::VmError))?
            .try_into()?;

        Ok(arr)
    }

    pub fn get_dict(&self, index: usize) -> crate::Result<&DictionaryObject> {
        let arr: &DictionaryObject = self
            .mem()
            .get(index)
            .ok_or(Error::from(ErrorKind::VmError))?
            .try_into()?;

        Ok(arr)
    }

    pub fn get_dict_mut(&mut self, index: usize) -> crate::Result<&mut DictionaryObject> {
        let arr: &mut DictionaryObject = self
            .mem_mut()
            .get_mut(index)
            .ok_or(Error::from(ErrorKind::VmError))?
            .try_into()?;

        Ok(arr)
    }

    pub fn mem(&self) -> &Container<Composite> {
        // TODO: Switch between local and global as needed
        &self.local_mem
    }

    pub fn mem_mut(&mut self) -> &mut Container<Composite> {
        // TODO: Switch between local and global as needed
        &mut self.local_mem
    }

    pub fn pop(&mut self) -> crate::Result<Object> {
        self.operand_stack
            .pop()
            .ok_or(Error::from(ErrorKind::StackUnderflow))
    }

    pub fn pop_dict(&mut self) -> crate::Result<&DictionaryObject> {
        match self.pop()? {
            Object::Dictionary(idx) => {
                let dict = self.get_dict(idx)?;

                Ok(dict)
            },
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }

    pub fn pop_int(&mut self) -> crate::Result<i32> {
        match self.pop()? {
            Object::Integer(i) => Ok(i),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        }
    }

    pub fn pop_usize(&mut self) -> crate::Result<usize> {
        match self.pop()? {
            Object::Integer(i) => {
                let u: usize = i.try_into().or(Err(Error::from(ErrorKind::RangeCheck)))?;

                Ok(u)
            },
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        }
    }

    pub fn push(&mut self, obj: Object) {
        self.operand_stack.push(obj);
    }
}

pub fn system_dict() -> Composite {
    type KeyOperatorPair = (&'static str, fn(&mut Context) -> crate::Result<()>);

    let ops: Vec<KeyOperatorPair> = vec![
        ("dup", dup),
        ("exch", exch),
        ("pop", |state| {
            state.pop()?;
            Ok(())
        }),
        ("copy", copy),
        ("roll", roll),
        ("index", index),
        ("mark", mark),
        ("clear", clear),
        ("count", count),
        ("counttomark", counttomark),
        ("cleartomark", cleartomark),
    ];

    let mut dict = HashMap::new();
    for (key, op) in ops {
        dict.insert(key.to_string(), Object::Operator(op));
    }

    let dict = DictionaryObject::new(dict, Access::ExecuteOnly, Mode::Executable);

    Composite::Dictionary(dict)
}
