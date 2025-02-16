use std::{collections::HashMap, io};

use crate::{
    object::{Access, Container, Object, PostScriptArray, PostScriptDictionary, PostScriptString},
    operators,
    rand::RandomNumberGenerator,
    Error, ErrorKind, Lexer,
};

pub struct Interpreter {
    pub arrays: Container<PostScriptArray>,
    pub dict_stack: Vec<usize>,
    pub dicts: Container<PostScriptDictionary>,
    pub execution_stack: Vec<Object>,
    pub is_packing: bool,
    pub operand_stack: Vec<Object>,
    pub rng: RandomNumberGenerator,
    pub strings: Container<PostScriptString>,
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut dicts = Container::default();

        let mut system_dict = PostScriptDictionary::from(operators::system_dict());
        system_dict.set_access(Access::ExecuteOnly);
        let system_dict_idx = dicts.insert(system_dict);

        let global_dict = dicts.insert(HashMap::new().into());
        let user_dict = dicts.insert(HashMap::new().into());

        Self {
            arrays: Container::default(),
            dict_stack: vec![system_dict_idx, global_dict, user_dict],
            execution_stack: Vec::default(),
            is_packing: false,
            operand_stack: Vec::default(),
            rng: RandomNumberGenerator::default(),
            strings: Container::default(),
            dicts,
        }
    }
}

impl Interpreter {
    pub fn evaluate<I>(&mut self, mut lexer: Lexer<I>) -> crate::Result<()>
    where
        I: Iterator<Item = char>,
    {
        while let Some(obj) = lexer.next_obj(&mut self.strings, &mut self.arrays) {
            let obj = obj?;

            if obj.is_procedure() {
                self.push(obj);
                continue;
            }

            self.execute_object(obj)?;
        }
        Ok(())
    }

    pub fn execute_object(&mut self, obj: Object) -> crate::Result<()> {
        match obj {
            Object::Integer(..) | Object::Real(..) | Object::Boolean(..) | Object::String(..) => {
                self.push(obj);
                Ok(())
            },
            Object::Operator(op) => op(self),
            Object::Procedure(idx) => {
                let proc = self.arrays.get(idx)?;

                for obj in proc.value().clone() {
                    self.execute_object(obj)?;
                }

                Ok(())
            },
            Object::Name(name) => {
                if name.starts_with('/') {
                    let name = name.trim_start_matches('/');
                    self.push(Object::Name(name.to_string()));
                    return Ok(());
                }

                let obj = self.search(name)?.clone();

                self.execute_object(obj)
            },
            _ => Err(Error::new(ErrorKind::Unregistered, "not implemented")),
        }
    }

    pub fn write_stack(&self, writer: &mut impl io::Write) -> io::Result<usize> {
        let mut count = 0;

        for obj in &self.operand_stack {
            count += writer.write(b" ")?;
            count += self.write_object(writer, obj)?;
        }

        Ok(count)
    }

    fn write_object(&self, writer: &mut impl io::Write, obj: &Object) -> io::Result<usize> {
        match obj {
            Object::Integer(i) => writer.write(i.to_string().as_bytes()),
            Object::Real(r) => {
                let is_whole_number = r.fract() == 0.0;
                if is_whole_number {
                    writer.write(format!("{:.1}", r).as_bytes())
                } else {
                    writer.write(r.to_string().as_bytes())
                }
            },
            Object::Boolean(b) => writer.write(b.to_string().as_bytes()),
            Object::String(idx) => {
                let string = format!(
                    "({})",
                    self.strings
                        .get(*idx)
                        .or(Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "missing string",
                        )))?
                        .value()
                );
                let output = string.as_bytes();

                writer.write(output)
            },
            Object::Array(idx) | Object::PackedArray(idx) => {
                let mut count = writer.write(b"[")?;

                let arr = self
                    .arrays
                    .get(*idx)
                    .or(Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "missing array",
                    )))?
                    .value();

                for obj in arr {
                    count += writer.write(b" ")?;
                    count += self.write_object(writer, obj)?;
                }

                count += writer.write(b" ]")?;

                Ok(count)
            },
            Object::Procedure(idx) => {
                let mut count = writer.write(b"{")?;

                let arr = self
                    .arrays
                    .get(*idx)
                    .or(Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "missing array",
                    )))?
                    .value();

                for obj in arr {
                    count += writer.write(b" ")?;
                    count += self.write_object(writer, obj)?;
                }

                count += writer.write(b" }")?;

                Ok(count)
            },
            Object::Dictionary(idx) => {
                let mut count = writer.write(b"<<")?;

                let dict = self
                    .dicts
                    .get(*idx)
                    .or(Err(io::Error::new(io::ErrorKind::NotFound, "missing dict")))?
                    .value();

                for (key, value) in dict {
                    count += writer.write(b" ")?;

                    let key: Vec<u8> = key.bytes().collect();
                    count += writer.write(&key)?;

                    count += writer.write(b" ")?;
                    count += self.write_object(writer, value)?;
                }

                count += writer.write(b" >>")?;

                Ok(count)
            },
            Object::Name(name) => writer.write(name.as_bytes()),
            Object::Mark => writer.write(b"mark"),
            Object::Null => writer.write(b"null"),
            _ => Ok(0),
        }
    }

    pub fn search(&self, name: String) -> crate::Result<&Object> {
        for dict_idx in self.dict_stack.iter().rev() {
            if let Ok(dict) = self.dicts.get(*dict_idx) {
                if let Ok(obj) = dict.get(name.clone()) {
                    return Ok(obj);
                }
            }
        }

        Err(Error::new(ErrorKind::Undefined, name))
    }

    pub fn search_mut(&mut self, name: String) -> crate::Result<&mut Object> {
        let dict = self.dict_stack.iter().rev().find(|idx| {
            self.dicts
                .get(**idx)
                .is_ok_and(|dict| dict.value().contains_key(&name))
        });

        match dict {
            Some(idx) => {
                let obj = self.dicts.get_mut(*idx)?.get_mut(name)?;
                Ok(obj)
            },
            None => Err(Error::new(ErrorKind::Undefined, name)),
        }
    }

    pub fn push(&mut self, obj: Object) {
        self.operand_stack.push(obj);
    }

    pub fn pop_literal(&mut self) -> crate::Result<Object> {
        match self.operand_stack.pop() {
            Some(obj) => Ok(obj),
            None => Err(Error::from(ErrorKind::StackUnderflow)),
        }
    }

    pub fn pop(&mut self) -> crate::Result<Object> {
        let obj = self.pop_literal()?;

        if let Object::Name(name) = obj {
            return self.search(name).cloned();
        }

        Ok(obj)
    }

    pub fn pop_int(&mut self) -> crate::Result<i32> {
        match self.pop()? {
            Object::Integer(i) => Ok(i),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        }
    }

    pub fn pop_real(&mut self) -> crate::Result<f64> {
        match self.pop()? {
            Object::Integer(i) => Ok(f64::from(i)),
            Object::Real(r) => Ok(r),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected real")),
        }
    }

    pub fn pop_usize(&mut self) -> crate::Result<usize> {
        let i = self.pop_int()?;

        match i.try_into() {
            Ok(u) => Ok(u),
            Err(_) => Err(Error::from(ErrorKind::RangeCheck)),
        }
    }

    pub fn pop_array(&mut self) -> crate::Result<&PostScriptArray> {
        match self.pop()? {
            Object::Array(idx) => Ok(self.arrays.get(idx)?),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    pub fn pop_array_mut(&mut self) -> crate::Result<&mut PostScriptArray> {
        match self.pop()? {
            Object::Array(idx) => Ok(self.arrays.get_mut(idx)?),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    pub fn pop_bool(&mut self) -> crate::Result<bool> {
        match self.pop()? {
            Object::Boolean(b) => Ok(b),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected boolean")),
        }
    }

    pub fn pop_dict(&mut self) -> crate::Result<&PostScriptDictionary> {
        match self.pop()? {
            Object::Dictionary(idx) => Ok(self.dicts.get(idx)?),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }
}
