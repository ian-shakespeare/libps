use std::{collections::HashMap, io};

use crate::{
    access::Access,
    composite::{Composite, Mode},
    memory::VirtualMemory,
    object::Object,
    operators,
    rand::RandomNumberGenerator,
    value::Value,
    Error, ErrorKind, Lexer,
};

pub struct Interpreter {
    pub dict_stack: Vec<Composite>,
    pub execution_stack: Vec<Object>,
    pub is_packing: bool,
    pub mem: VirtualMemory<Value>,
    pub operand_stack: Vec<Object>,
    pub rng: RandomNumberGenerator,
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut mem = VirtualMemory::new();

        let system_dict: Value = operators::system_dict().into();
        let key = mem.insert(system_dict);
        let system_dict = Composite {
            access: Access::ExecuteOnly,
            mode: Mode::Executable,
            key,
        };

        let key = mem.insert(HashMap::new());
        let global_dict = Composite {
            access: Access::Unlimited,
            mode: Mode::Executable,
            key,
        };

        let key = mem.insert(HashMap::new());
        let user_dict = Composite {
            access: Access::Unlimited,
            mode: Mode::Executable,
            key,
        };

        Self {
            dict_stack: vec![system_dict, global_dict, user_dict],
            execution_stack: Vec::default(),
            is_packing: false,
            operand_stack: Vec::default(),
            rng: RandomNumberGenerator::default(),
            mem,
        }
    }
}

impl Interpreter {
    pub fn evaluate<I: Into<Lexer<C>>, C>(&mut self, input: I) -> crate::Result<()>
    where
        C: Iterator<Item = char>,
    {
        let mut lexer: Lexer<C> = input.into();
        while let Some(obj) = lexer.next_obj(&mut self.mem) {
            let obj = obj?;

            if obj.is_array() {
                let Composite { mode, .. } = obj.into_composite()?;

                if mode == Mode::Literal {
                    self.push(obj);
                    continue;
                }
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
            Object::Array(comp) => {
                if comp.mode == Mode::Executable {
                    let arr: &Vec<Object> = self.mem.get(comp.key)?.try_into()?;
                    for obj in arr.clone() {
                        self.execute_object(obj)?;
                    }

                    Ok(())
                } else {
                    self.push(Object::Array(comp));

                    Ok(())
                }
            },
            Object::Name(name) => {
                if name.mode == Mode::Literal {
                    let name = name.value().trim_start_matches('/');
                    self.push(Object::Name(name.into()));

                    Ok(())
                } else {
                    let obj = self.find(name.value())?.clone();

                    self.execute_object(obj)
                }
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
            Object::String(Composite { key, .. }) => {
                let string: &str = self
                    .mem
                    .get(*key)
                    .or(Err(io::Error::from(io::ErrorKind::NotFound)))?
                    .try_into()
                    .or(Err(io::Error::from(io::ErrorKind::InvalidData)))?;
                let string = format!("({})", string);
                let output = string.as_bytes();

                writer.write(output)
            },
            Object::Array(Composite { access, key, .. })
            | Object::PackedArray(Composite { access, key, .. }) => {
                let (left_delim, right_delim) = if access.is_exec_only() {
                    (b"{", b" }")
                } else {
                    (b"[", b" ]")
                };
                let mut count = writer.write(left_delim)?;

                let arr: &Vec<Object> = self
                    .mem
                    .get(*key)
                    .or(Err(io::Error::from(io::ErrorKind::NotFound)))?
                    .try_into()
                    .or(Err(io::Error::from(io::ErrorKind::InvalidData)))?;

                for obj in arr {
                    count += writer.write(b" ")?;
                    count += self.write_object(writer, obj)?;
                }

                count += writer.write(right_delim)?;

                Ok(count)
            },
            Object::Dictionary(Composite { key, .. }) => {
                let mut count = writer.write(b"<<")?;

                let dict: &HashMap<String, Object> = self
                    .mem
                    .get(*key)
                    .or(Err(io::Error::from(io::ErrorKind::NotFound)))?
                    .try_into()
                    .or(Err(io::Error::from(io::ErrorKind::InvalidData)))?;

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
            Object::Name(name) => writer.write(name.value().as_bytes()),
            Object::Mark => writer.write(b"mark"),
            Object::Null => writer.write(b"null"),
            _ => Ok(0),
        }
    }

    pub fn stringify(&self, obj: &Object) -> crate::Result<String> {
        match obj {
            Object::Integer(i) => Ok(i.to_string()),
            Object::Real(r) => Ok(r.to_string()),
            Object::String(Composite { key, .. }) => {
                let s: &str = self.mem.get(*key)?.try_into()?;

                Ok(s.to_string())
            },
            Object::Name(name) => Ok(name.value().to_string()),
            _ => Err(Error::new(
                ErrorKind::Unregistered,
                "cannot stringify object",
            )),
        }
    }

    pub fn find_dict(&self, name: &str) -> crate::Result<usize> {
        for Composite { key, .. } in self.dict_stack.iter().rev() {
            let dict: &HashMap<String, Object> = self.mem.get(*key)?.try_into()?;
            if dict.contains_key(name) {
                return Ok(*key);
            }
        }

        Err(Error::new(ErrorKind::Undefined, name))
    }

    pub fn find(&self, name: &str) -> crate::Result<&Object> {
        for Composite { key, .. } in self.dict_stack.iter().rev() {
            let dict: &HashMap<String, Object> = self.mem.get(*key)?.try_into()?;
            if let Some(obj) = dict.get(name) {
                return Ok(obj);
            }
        }

        Err(Error::new(ErrorKind::Undefined, name))
    }

    pub fn find_mut(&mut self, name: &str) -> crate::Result<&mut Object> {
        let key = self.find_dict(name)?;

        let dict: &mut HashMap<String, Object> = self.mem.get_mut(key)?.try_into()?;
        let obj = dict
            .get_mut(name)
            .ok_or(Error::from(ErrorKind::Undefined))?;

        Ok(obj)
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
            return self.find(name.value()).cloned();
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

    pub fn pop_array(&mut self) -> crate::Result<Composite> {
        match self.pop()? {
            Object::Array(comp) => Ok(comp),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    pub fn pop_bool(&mut self) -> crate::Result<bool> {
        match self.pop()? {
            Object::Boolean(b) => Ok(b),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected boolean")),
        }
    }

    pub fn pop_dict(&mut self) -> crate::Result<Composite> {
        match self.pop()? {
            Object::Dictionary(comp) => Ok(comp),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }
}
