use std::collections::HashMap;

use crate::{
    array_operator::*,
    container::Container,
    math_operator::*,
    object::{Access, Composite, DictionaryObject, Mode, Object},
    rand::RandomNumberGenerator,
    relational_operator::*,
    stack_operator::*,
    ArrayObject, Error, ErrorKind, StringObject,
};

#[allow(dead_code)]
pub struct Context {
    pub rng: RandomNumberGenerator,
    pub operand_stack: Vec<Object>,
    pub is_packing: bool,
    local_mem: Container<Composite>,
    global_mem: Container<Composite>,
    dict_stack: Vec<usize>,
}

impl Default for Context {
    fn default() -> Self {
        let mut local_mem: Container<Composite> = Container::default();
        let system_dict = local_mem.insert(system_dict());
        let global_dict = local_mem.insert(DictionaryObject::new(
            HashMap::new(),
            Access::Unlimited,
            Mode::Executable,
        ));
        let user_dict = local_mem.insert(DictionaryObject::new(
            HashMap::new(),
            Access::Unlimited,
            Mode::Executable,
        ));

        Self {
            rng: RandomNumberGenerator::default(),
            operand_stack: Vec::default(),
            global_mem: Container::default(),
            is_packing: false,
            dict_stack: vec![system_dict, global_dict, user_dict],
            local_mem,
        }
    }
}

impl<'a> Context {
    pub fn with_test_utils() -> Self {
        let mut ctx = Context::default();

        let global_dict = ctx
            .get_dict_mut(ctx.dict_stack[1])
            .expect("failed to get global dict");
        global_dict.insert(
            "assert".to_string(),
            Object::Operator(|ctx| {
                assert!(ctx.pop_bool()?);

                Ok(())
            }),
        );

        ctx
    }

    pub fn find_def<S>(&'a self, key: S) -> crate::Result<&'a Object>
    where
        S: Into<&'a str>,
    {
        let key: &str = key.into();

        for idx in self.dict_stack.iter().rev() {
            if let Some(comp) = self.mem().get(*idx) {
                let dict: &DictionaryObject = comp.try_into()?;

                if dict.get(key).is_ok() {
                    return dict.get(key);
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

    pub fn get_string(&self, index: usize) -> crate::Result<&StringObject> {
        let string: &StringObject = self
            .mem()
            .get(index)
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
        self.operand_stack
            .pop()
            .ok_or(Error::from(ErrorKind::StackUnderflow))
    }

    pub fn pop_array(&mut self) -> crate::Result<&ArrayObject> {
        let idx = self.pop()?.into_index()?;

        self.get_array(idx)
    }

    pub fn pop_array_mut(&mut self) -> crate::Result<&mut ArrayObject> {
        let idx = self.pop()?.into_index()?;

        self.get_array_mut(idx)
    }

    pub fn pop_bool(&mut self) -> crate::Result<bool> {
        match self.pop()? {
            Object::Boolean(b) => Ok(b),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected boolean")),
        }
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
        self.pop()?.into_int()
    }

    pub fn pop_real(&mut self) -> crate::Result<f64> {
        self.pop()?.into_real()
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
        ("eq", eq),
    ];

    let mut dict = HashMap::new();
    for (key, op) in ops {
        dict.insert(key.to_string(), Object::Operator(op));
    }

    let dict = DictionaryObject::new(dict, Access::ExecuteOnly, Mode::Executable);

    Composite::Dictionary(dict)
}
