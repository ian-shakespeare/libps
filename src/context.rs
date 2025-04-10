use std::collections::HashMap;

use crate::{
    container::Container,
    object::{Access, Composite, DictionaryObject, Mode, Object, OperatorObject},
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
        ("dup", Object::Operator(OperatorObject::Dup)),
        ("exch", Object::Operator(OperatorObject::Exch)),
        ("pop", Object::Operator(OperatorObject::Pop)),
        ("copy", Object::Operator(OperatorObject::Copy)),
        ("roll", Object::Operator(OperatorObject::Roll)),
        ("index", Object::Operator(OperatorObject::Index)),
        ("mark", Object::Mark),
        ("clear", Object::Operator(OperatorObject::Clear)),
        ("count", Object::Operator(OperatorObject::Count)),
        ("counttomark", Object::Operator(OperatorObject::Counttomark)),
        ("cleartomark", Object::Operator(OperatorObject::Cleartomark)),
        ("add", Object::Operator(OperatorObject::Add)),
        ("div", Object::Operator(OperatorObject::Div)),
        ("idiv", Object::Operator(OperatorObject::Idiv)),
        ("mod", Object::Operator(OperatorObject::Mod)),
        ("mul", Object::Operator(OperatorObject::Mul)),
        ("sub", Object::Operator(OperatorObject::Sub)),
        ("abs", Object::Operator(OperatorObject::Abs)),
        ("neg", Object::Operator(OperatorObject::Neg)),
        ("ceiling", Object::Operator(OperatorObject::Ceiling)),
        ("floor", Object::Operator(OperatorObject::Floor)),
        ("round", Object::Operator(OperatorObject::Round)),
        ("truncate", Object::Operator(OperatorObject::Truncate)),
        ("sqrt", Object::Operator(OperatorObject::Sqrt)),
        ("atan", Object::Operator(OperatorObject::Atan)),
        ("cos", Object::Operator(OperatorObject::Cos)),
        ("sin", Object::Operator(OperatorObject::Sin)),
        ("exp", Object::Operator(OperatorObject::Exp)),
        ("ln", Object::Operator(OperatorObject::Ln)),
        ("log", Object::Operator(OperatorObject::Log)),
        ("rand", Object::Operator(OperatorObject::Rand)),
        ("srand", Object::Operator(OperatorObject::Srand)),
        ("rrand", Object::Operator(OperatorObject::Rrand)),
        ("array", Object::Operator(OperatorObject::Array)),
        ("[", Object::Mark),
        ("]", Object::Operator(OperatorObject::EndArray)),
        ("length", Object::Operator(OperatorObject::Length)),
        ("get", Object::Operator(OperatorObject::Get)),
        ("put", Object::Operator(OperatorObject::Put)),
        ("getinterval", Object::Operator(OperatorObject::Getinterval)),
        ("putinterval", Object::Operator(OperatorObject::Putinterval)),
        ("astore", Object::Operator(OperatorObject::Astore)),
        ("aload", Object::Operator(OperatorObject::Aload)),
        ("forall", Object::Operator(OperatorObject::Forall)),
        ("packedarray", Object::Operator(OperatorObject::Packedarray)),
        ("setpacking", Object::Operator(OperatorObject::Setpacking)),
        (
            "currentpacking",
            Object::Operator(OperatorObject::Currentpacking),
        ),
        ("dict", Object::Operator(OperatorObject::Dict)),
        ("<<", Object::Mark),
        (">>", Object::Operator(OperatorObject::EndDict)),
        ("maxlength", Object::Operator(OperatorObject::Maxlength)),
        ("begin", Object::Operator(OperatorObject::Begin)),
        ("end", Object::Operator(OperatorObject::End)),
        ("def", Object::Operator(OperatorObject::Def)),
        ("load", Object::Operator(OperatorObject::Load)),
        ("store", Object::Operator(OperatorObject::Store)),
        ("undef", Object::Operator(OperatorObject::Undef)),
        ("known", Object::Operator(OperatorObject::Known)),
        ("where", Object::Operator(OperatorObject::Where)),
        ("currentdict", Object::Operator(OperatorObject::Currentdict)),
        ("errordict", Object::Dictionary(error_dict_idx)),
        ("$error", Object::Dictionary(error_info_idx)),
        (
            "countdictstack",
            Object::Operator(OperatorObject::Countdictstack),
        ),
        ("eq", Object::Operator(OperatorObject::Eq)),
        ("true", Object::Boolean(true)),
        ("false", Object::Boolean(false)),
        ("type", Object::Operator(OperatorObject::Type)),
        ("null", Object::Null),
        ("handleerror", Object::Operator(OperatorObject::Handleerror)),
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
            OperatorObject::Dictstackunderflow,
        ),
        (ErrorKind::InvalidAccess, OperatorObject::Invalidaccess),
        (ErrorKind::IoError, OperatorObject::Ioerror),
        (ErrorKind::LimitCheck, OperatorObject::Limitcheck),
        (ErrorKind::RangeCheck, OperatorObject::Rangecheck),
        (ErrorKind::StackUnderflow, OperatorObject::Stackunderflow),
        (ErrorKind::SyntaxError, OperatorObject::Syntaxerror),
        (ErrorKind::TypeCheck, OperatorObject::Typecheck),
        (ErrorKind::Undefined, OperatorObject::Undefined),
        (ErrorKind::UndefinedResult, OperatorObject::Undefinedresult),
        (ErrorKind::UnmatchedMark, OperatorObject::Unmatchedmark),
        (ErrorKind::Unregistered, OperatorObject::Unregistered),
        (ErrorKind::VmError, OperatorObject::Vmerror),
    ];

    definitions.into_iter().fold(
        DictionaryObject::new(HashMap::new(), Access::Unlimited, Mode::default()),
        |mut dict, (key, op)| {
            let key: &str = key.into();
            dict.insert(key, Object::Operator(op));
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
