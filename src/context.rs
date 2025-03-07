use std::collections::HashMap;

use crate::{
    array_operators::*,
    container::Container,
    debug_operators::*,
    dict_operators::*,
    math_operators::*,
    misc_operators::*,
    object::{Access, Composite, DictionaryObject, Mode, Object},
    rand::RandomNumberGenerator,
    relational_operators::*,
    stack_operators::*,
    type_operators::*,
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

        let system_dict = system_operators().fold(HashMap::new(), |mut dict, (key, op)| {
            dict.insert(key.to_string(), Object::Operator(op));
            dict
        });
        let system_idx = local_mem.insert(DictionaryObject::new(
            system_dict,
            Access::ExecuteOnly,
            Mode::Executable,
        ));

        let global_idx = local_mem.insert(DictionaryObject::new(
            HashMap::new(),
            Access::Unlimited,
            Mode::Executable,
        ));

        let user_idx = local_mem.insert(DictionaryObject::new(
            HashMap::new(),
            Access::Unlimited,
            Mode::Executable,
        ));

        Self {
            rng: RandomNumberGenerator::default(),
            operand_stack: Vec::default(),
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

        for (key, op) in debug_operators() {
            global_dict.insert(key.to_string(), Object::Operator(op));
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

type KeyOperatorPair = (&'static str, fn(&mut Context) -> crate::Result<()>);

fn system_operators() -> impl Iterator<Item = KeyOperatorPair> {
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
        ("add", |state| {
            arithmetic(state, i32::checked_add, |a: f64, b: f64| a + b)
        }),
        ("div", |state| {
            arithmetic(state, |_, _| None, |a: f64, b: f64| a / b)
        }),
        ("idiv", idiv),
        ("imod", imod),
        ("mul", |state| {
            arithmetic(state, i32::checked_mul, |a: f64, b: f64| a * b)
        }),
        ("sub", |state| {
            arithmetic(state, i32::checked_sub, |a: f64, b: f64| a - b)
        }),
        ("abs", |state| num_unary(state, i32::checked_abs, f64::abs)),
        ("neg", |state| {
            num_unary(state, i32::checked_neg, |a: f64| -1.0 * a)
        }),
        ("ceiling", |state| {
            num_unary(state, |a: i32| Some(a), f64::ceil)
        }),
        ("floor", |state| {
            num_unary(state, |a: i32| Some(a), f64::floor)
        }),
        ("round", |state| {
            num_unary(state, |a: i32| Some(a), f64::round)
        }),
        ("truncate", |state| {
            num_unary(state, |a: i32| Some(a), f64::trunc)
        }),
        ("sqrt", |state| real_unary(state, f64::sqrt)),
        ("atan", atan),
        ("cos", cos),
        ("sin", sin),
        ("exp", |state| {
            arithmetic(state, |_, _| None, |base: f64, exp: f64| base.powf(exp))
        }),
        ("ln", |state| real_unary(state, f64::ln)),
        ("log", |state| real_unary(state, f64::log10)),
        ("rand", rand),
        ("srand", srand),
        ("rrand", rrand),
        ("array", array),
        ("[", mark),
        ("]", endarray),
        ("length", length),
        ("get", get),
        ("put", put),
        ("getinterval", getinterval),
        ("putinterval", putinterval),
        ("astore", astore),
        ("aload", aload),
        ("forall", forall),
        ("packedarray", packedarray),
        ("setpacking", setpacking),
        ("currentpacking", currentpacking),
        ("dict", dict),
        ("<<", mark),
        (">>", enddict),
        ("maxlength", maxlength),
        ("begin", begin),
        ("end", end),
        ("def", def),
        ("load", load),
        ("store", store),
        ("undef", undef),
        ("known", known),
        ("where", wheredef),
        ("eq", eq),
        ("true", pushtrue),
        ("false", pushfalse),
        ("type", gettype),
        ("null", null),
    ];

    ops.into_iter()
}

fn debug_operators() -> impl Iterator<Item = KeyOperatorPair> {
    let ops: Vec<KeyOperatorPair> = vec![
        ("assert", assert),
        ("asserteq", asserteq),
        ("assertne", assertne),
        ("assertdeepeq", assertdeepeq),
    ];

    ops.into_iter()
}
