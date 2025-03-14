use std::collections::HashMap;

use crate::{
    container::Container,
    object::{Access, Composite, DictionaryObject, Mode, Object},
    operators::*,
    rand::RandomNumberGenerator,
    ArrayObject, Error, ErrorKind, StringObject,
};

#[allow(dead_code)]
pub struct Context {
    pub rng: RandomNumberGenerator,
    pub operand_stack: Vec<Object>,
    pub is_packing: bool,
    local_mem: Container<Composite>,
    global_mem: Container<Composite>,
    pub dict_stack: Vec<usize>,
}

impl Default for Context {
    fn default() -> Self {
        let mut local_mem: Container<Composite> = Container::default();

        let system_dict = system_dict(&mut local_mem);
        let system_idx = local_mem.insert(system_dict);

        let global_idx = local_mem.insert(DictionaryObject::new(
            HashMap::new(),
            Access::Unlimited,
            Mode::default(),
        ));

        let user_idx = local_mem.insert(DictionaryObject::new(
            HashMap::new(),
            Access::Unlimited,
            Mode::default(),
        ));

        Self {
            rng: RandomNumberGenerator::default(),
            operand_stack: Vec::new(),
            global_mem: Container::default(),
            is_packing: false,
            dict_stack: vec![system_idx, global_idx, user_idx],
            local_mem,
        }
    }
}

impl Context {
    pub fn with_debug_utils() -> Self {
        let mut ctx = Context::default();

        let global_dict = ctx
            .get_dict_mut(ctx.dict_stack[1])
            .expect("failed to get global dict");

        for (key, op) in debug_dict().into_iter() {
            global_dict.insert(key, op);
        }

        ctx
    }

    pub fn find_def<S>(&self, key: S) -> crate::Result<&Object>
    where
        S: AsRef<str>,
    {
        let key = key.as_ref();

        for idx in self.dict_stack.iter().rev() {
            if let Ok(dict) = self.get_dict(*idx) {
                if dict.contains_key(key) {
                    return dict.get(key);
                }
            }
        }

        Err(Error::new(ErrorKind::Undefined, key))
    }

    pub fn find_dict<S>(&self, key: S) -> crate::Result<&DictionaryObject>
    where
        S: AsRef<str>,
    {
        let key = key.as_ref();

        for idx in self.dict_stack.iter().rev() {
            if self.get_dict(*idx).is_ok_and(|d| d.contains_key(key)) {
                return self.get_dict(*idx);
            }
        }

        Err(Error::new(ErrorKind::Undefined, key))
    }

    pub fn find_dict_mut<S>(&mut self, key: S) -> crate::Result<&mut DictionaryObject>
    where
        S: AsRef<str>,
    {
        let key = key.as_ref();

        for idx in self.dict_stack.iter().rev() {
            if self.get_dict(*idx).is_ok_and(|d| d.contains_key(key)) {
                return self.get_dict_mut(*idx);
            }
        }

        Err(Error::new(ErrorKind::Undefined, key))
    }

    pub fn find_index<S>(&self, key: S) -> crate::Result<usize>
    where
        S: AsRef<str>,
    {
        let key = key.as_ref();

        for idx in self.dict_stack.iter().rev() {
            if self.get_dict(*idx).is_ok_and(|d| d.contains_key(key)) {
                return Ok(*idx);
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

    pub fn get_string(&self, index: usize) -> crate::Result<&StringObject> {
        let string: &StringObject = self
            .mem()
            .get(index)
            .ok_or(Error::from(ErrorKind::VmError))?
            .try_into()?;

        Ok(string)
    }

    pub fn get_string_mut(&mut self, index: usize) -> crate::Result<&mut StringObject> {
        let string: &mut StringObject = self
            .mem_mut()
            .get_mut(index)
            .ok_or(Error::from(ErrorKind::VmError))?
            .try_into()?;

        Ok(string)
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
        let obj = self
            .operand_stack
            .pop()
            .ok_or(Error::from(ErrorKind::StackUnderflow))?;

        if obj.is_name() && obj.mode(self)? == Mode::Executable {
            let name = obj.into_name()?;
            return self.find_def(name).cloned();
        }

        Ok(obj)
    }

    pub fn pop_array(&mut self) -> crate::Result<&ArrayObject> {
        match self.pop()? {
            Object::Array(idx) => self.get_array(idx),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    pub fn pop_array_mut(&mut self) -> crate::Result<&mut ArrayObject> {
        match self.pop()? {
            Object::Array(idx) => self.get_array_mut(idx),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    pub fn pop_bool(&mut self) -> crate::Result<bool> {
        match self.pop()? {
            Object::Boolean(b) => Ok(b),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected boolean")),
        }
    }

    pub fn pop_dict(&mut self) -> crate::Result<&DictionaryObject> {
        match self.pop()? {
            Object::Dictionary(idx) => self.get_dict(idx),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }

    pub fn pop_dict_mut(&mut self) -> crate::Result<&mut DictionaryObject> {
        match self.pop()? {
            Object::Dictionary(idx) => self.get_dict_mut(idx),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }

    pub fn pop_int(&mut self) -> crate::Result<i32> {
        self.pop()?.into_int()
    }

    pub fn pop_real(&mut self) -> crate::Result<f64> {
        self.pop()?.into_real()
    }

    pub fn pop_string(&mut self) -> crate::Result<&StringObject> {
        match self.pop()? {
            Object::String(idx) => self.get_string(idx),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }

    pub fn pop_string_mut(&mut self) -> crate::Result<&mut StringObject> {
        match self.pop()? {
            Object::String(idx) => self.get_string_mut(idx),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }

    pub fn pop_usize(&mut self) -> crate::Result<usize> {
        self.pop()?.into_usize()
    }

    pub fn push(&mut self, obj: Object) {
        self.operand_stack.push(obj);
    }

    pub fn stringify(&self, obj: &Object) -> crate::Result<String> {
        match obj {
            Object::Integer(i) => Ok(i.to_string()),
            Object::Real(r) => Ok(r.to_string()),
            Object::String(idx) => {
                let string: &str = self.get_string(*idx)?.into();

                Ok(string.to_string())
            },
            Object::Name(name) => {
                let name: &str = name.into();

                Ok(name.to_string())
            },
            _ => Err(Error::new(
                ErrorKind::Unregistered,
                "cannot stringify object",
            )),
        }
    }
}

fn system_dict(mem: &mut Container<Composite>) -> DictionaryObject {
    let error_dict_idx = mem.insert(error_dict());
    let error_info_idx = mem.insert(error_info_dict());

    let definitions = [
        ("dup", Object::Operator(dup)),
        ("exch", Object::Operator(exch)),
        (
            "pop",
            Object::Operator(|ctx| {
                ctx.pop()?;
                Ok(())
            }),
        ),
        ("copy", Object::Operator(copy)),
        ("roll", Object::Operator(roll)),
        ("index", Object::Operator(index)),
        ("mark", Object::Operator(mark)),
        ("clear", Object::Operator(clear)),
        ("count", Object::Operator(count)),
        ("counttomark", Object::Operator(counttomark)),
        ("cleartomark", Object::Operator(cleartomark)),
        (
            "add",
            Object::Operator(|ctx| arithmetic(ctx, i32::checked_add, |a: f64, b: f64| a + b)),
        ),
        (
            "div",
            Object::Operator(|ctx| arithmetic(ctx, |_, _| None, |a: f64, b: f64| a / b)),
        ),
        ("idiv", Object::Operator(idiv)),
        ("mod", Object::Operator(imod)),
        (
            "mul",
            Object::Operator(|ctx| arithmetic(ctx, i32::checked_mul, |a: f64, b: f64| a * b)),
        ),
        (
            "sub",
            Object::Operator(|ctx| arithmetic(ctx, i32::checked_sub, |a: f64, b: f64| a - b)),
        ),
        (
            "abs",
            Object::Operator(|ctx| num_unary(ctx, i32::checked_abs, f64::abs)),
        ),
        (
            "neg",
            Object::Operator(|ctx| num_unary(ctx, i32::checked_neg, |a: f64| -1.0 * a)),
        ),
        (
            "ceiling",
            Object::Operator(|ctx| num_unary(ctx, |a: i32| Some(a), f64::ceil)),
        ),
        (
            "floor",
            Object::Operator(|ctx| num_unary(ctx, |a: i32| Some(a), f64::floor)),
        ),
        ("round", Object::Operator(round)),
        (
            "truncate",
            Object::Operator(|ctx| num_unary(ctx, |a: i32| Some(a), f64::trunc)),
        ),
        ("sqrt", Object::Operator(sqrt)),
        ("atan", Object::Operator(atan)),
        ("cos", Object::Operator(cos)),
        ("sin", Object::Operator(sin)),
        (
            "exp",
            Object::Operator(|ctx| {
                arithmetic(ctx, |_, _| None, |base: f64, exp: f64| base.powf(exp))
            }),
        ),
        ("ln", Object::Operator(ln)),
        ("log", Object::Operator(log)),
        ("rand", Object::Operator(rand)),
        ("srand", Object::Operator(srand)),
        ("rrand", Object::Operator(rrand)),
        ("array", Object::Operator(array)),
        ("[", Object::Operator(mark)),
        ("]", Object::Operator(endarray)),
        ("length", Object::Operator(length)),
        ("get", Object::Operator(get)),
        ("put", Object::Operator(put)),
        ("getinterval", Object::Operator(getinterval)),
        ("putinterval", Object::Operator(putinterval)),
        ("astore", Object::Operator(astore)),
        ("aload", Object::Operator(aload)),
        ("forall", Object::Operator(forall)),
        ("packedarray", Object::Operator(packedarray)),
        ("setpacking", Object::Operator(setpacking)),
        ("currentpacking", Object::Operator(currentpacking)),
        ("dict", Object::Operator(dict)),
        ("<<", Object::Operator(mark)),
        (">>", Object::Operator(enddict)),
        ("maxlength", Object::Operator(maxlength)),
        ("begin", Object::Operator(begin)),
        ("end", Object::Operator(end)),
        ("def", Object::Operator(def)),
        ("load", Object::Operator(load)),
        ("store", Object::Operator(store)),
        ("undef", Object::Operator(undef)),
        ("known", Object::Operator(known)),
        ("where", Object::Operator(wheredef)),
        ("currentdict", Object::Operator(currentdict)),
        ("errordict", Object::Dictionary(error_dict_idx)),
        ("$error", Object::Dictionary(error_info_idx)),
        ("eq", Object::Operator(eq)),
        ("true", Object::Boolean(true)),
        ("false", Object::Boolean(false)),
        ("type", Object::Operator(gettype)),
        ("null", Object::Operator(null)),
        ("handleerror", Object::Operator(handleerror)),
    ];

    definitions.into_iter().fold(
        DictionaryObject::new(HashMap::new(), Access::ExecuteOnly, Mode::default()),
        |mut dict, (key, obj)| {
            dict.insert(key, obj);
            dict
        },
    )
}

fn error_dict() -> DictionaryObject {
    let definitions = [
        (
            ErrorKind::DictStackUnderflow,
            Object::Operator(dictstackunderflow),
        ),
        (ErrorKind::InvalidAccess, Object::Operator(invalidaccess)),
        (ErrorKind::IoError, Object::Operator(ioerror)),
        (ErrorKind::LimitCheck, Object::Operator(limitcheck)),
        (ErrorKind::RangeCheck, Object::Operator(rangecheck)),
        (ErrorKind::StackUnderflow, Object::Operator(stackunderflow)),
        (ErrorKind::SyntaxError, Object::Operator(syntaxerror)),
        (ErrorKind::TypeCheck, Object::Operator(typecheck)),
        (ErrorKind::Undefined, Object::Operator(undefined)),
        (
            ErrorKind::UndefinedResult,
            Object::Operator(undefinedresult),
        ),
        (ErrorKind::UnmatchedMark, Object::Operator(unmatchedmark)),
        (ErrorKind::Unregistered, Object::Operator(unregistered)),
        (ErrorKind::VmError, Object::Operator(vmerror)),
    ];

    definitions.into_iter().fold(
        DictionaryObject::new(HashMap::new(), Access::Unlimited, Mode::default()),
        |mut dict, (key, obj)| {
            let key: &str = key.into();
            dict.insert(key, obj);
            dict
        },
    )
}

fn error_info_dict() -> DictionaryObject {
    let definitions = [
        ("newerror", Object::Boolean(false)),
        ("errorname", Object::Null),
        ("command", Object::Null),
        ("errorinfo", Object::Null),
        ("ostack", Object::Null),
        ("estack", Object::Null),
        ("dstack", Object::Null),
        // TODO: actually record all stacks and set this to true
        ("recordstacks", Object::Boolean(false)),
        ("binary", Object::Boolean(false)),
    ];

    definitions.into_iter().fold(
        DictionaryObject::new(HashMap::new(), Access::Unlimited, Mode::default()),
        |mut dict, (key, obj)| {
            dict.insert(key, obj);
            dict
        },
    )
}

fn debug_dict() -> DictionaryObject {
    let definitions = [
        ("assert", Object::Operator(assert)),
        ("asserteq", Object::Operator(asserteq)),
        ("assertne", Object::Operator(assertne)),
        ("assertdeepeq", Object::Operator(assertdeepeq)),
        ("asserterror", Object::Operator(asserterror)),
        ("assertnear", Object::Operator(assertnear)),
        ("assertgt", Object::Operator(assertgt)),
        ("assertlt", Object::Operator(assertlt)),
    ];

    definitions.into_iter().fold(
        DictionaryObject::new(HashMap::new(), Access::Unlimited, Mode::default()),
        |mut dict, (key, obj)| {
            dict.insert(key, obj);
            dict
        },
    )
}
