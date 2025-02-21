use std::{collections::HashMap, io};

use crate::{
    composite::{Access, Composite},
    memory::VirtualMemory,
    object::Object,
    operators,
    rand::RandomNumberGenerator,
    Error, ErrorKind, Lexer,
};

pub struct Interpreter {
    pub dict_stack: Vec<usize>,
    pub execution_stack: Vec<Object>,
    pub is_packing: bool,
    pub mem: VirtualMemory,
    pub operand_stack: Vec<Object>,
    pub rng: RandomNumberGenerator,
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut mem = VirtualMemory::default();

        let mut system_dict: Composite = operators::system_dict().into();
        system_dict.access = Access::ExecuteOnly;
        let system_dict_idx = mem.insert(system_dict);

        let global_dict = mem.insert(HashMap::new());
        let user_dict = mem.insert(HashMap::new());

        Self {
            dict_stack: vec![system_dict_idx, global_dict, user_dict],
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
        let mut lexer = input.into();
        while let Some(obj) = lexer.next_obj(&mut self.mem) {
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
                let proc = self.mem.get(idx)?;

                for obj in proc.array()?.clone() {
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

                let obj = self.find(name)?.clone();

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
                    self.mem.get_string(*idx).or(Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "missing string",
                    )))?
                );
                let output = string.as_bytes();

                writer.write(output)
            },
            Object::Array(idx) | Object::PackedArray(idx) => {
                let mut count = writer.write(b"[")?;

                let arr = self.mem.get_array(*idx).or(Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "missing array",
                )))?;

                for obj in arr {
                    count += writer.write(b" ")?;
                    count += self.write_object(writer, obj)?;
                }

                count += writer.write(b" ]")?;

                Ok(count)
            },
            Object::Procedure(idx) => {
                let mut count = writer.write(b"{")?;

                let arr = self.mem.get_array(*idx).or(Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "missing array",
                )))?;

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
                    .mem
                    .get_dict(*idx)
                    .or(Err(io::Error::new(io::ErrorKind::NotFound, "missing dict")))?;

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

    pub fn stringify(&self, obj: &Object) -> crate::Result<String> {
        match obj {
            Object::Integer(i) => Ok(i.to_string()),
            Object::Real(r) => Ok(r.to_string()),
            Object::String(idx) => Ok(self.mem.get_string(*idx)?.to_string()),
            Object::Name(name) => Ok(name.to_string()),
            _ => Err(Error::new(
                ErrorKind::Unregistered,
                "cannot stringify object",
            )),
        }
    }

    pub fn find_dict(&self, name: String) -> crate::Result<usize> {
        for dict_idx in self.dict_stack.iter().rev() {
            if self.mem.get_dict(*dict_idx)?.get(&name).is_some() {
                return Ok(*dict_idx);
            }
        }

        Err(Error::new(ErrorKind::Undefined, name))
    }

    pub fn find(&self, name: String) -> crate::Result<&Object> {
        for dict_idx in self.dict_stack.iter().rev() {
            if let Ok(dict) = self.mem.get_dict(*dict_idx) {
                if let Some(obj) = dict.get(&name) {
                    return Ok(obj);
                }
            }
        }

        Err(Error::new(ErrorKind::Undefined, name))
    }

    pub fn find_mut(&mut self, name: String) -> crate::Result<&mut Object> {
        let dict = self.dict_stack.iter().rev().find(|idx| {
            self.mem
                .get_dict(**idx)
                .is_ok_and(|dict| dict.contains_key(&name))
        });

        match dict {
            Some(idx) => {
                let obj = self
                    .mem
                    .get_dict_mut(*idx)?
                    .get_mut(&name)
                    .ok_or(Error::from(ErrorKind::Undefined))?;

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
            return self.find(name).cloned();
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

    pub fn pop_array(&mut self) -> crate::Result<&Composite> {
        match self.pop()? {
            Object::Array(idx) => Ok(self.mem.get(idx)?),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    pub fn pop_array_mut(&mut self) -> crate::Result<&mut Composite> {
        match self.pop()? {
            Object::Array(idx) => Ok(self.mem.get_mut(idx)?),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    pub fn pop_bool(&mut self) -> crate::Result<bool> {
        match self.pop()? {
            Object::Boolean(b) => Ok(b),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected boolean")),
        }
    }

    pub fn pop_dict(&mut self) -> crate::Result<&Composite> {
        match self.pop()? {
            Object::Dictionary(idx) => Ok(self.mem.get(idx)?),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }

    pub fn pop_dict_mut(&mut self) -> crate::Result<&mut Composite> {
        match self.pop()? {
            Object::Dictionary(idx) => Ok(self.mem.get_mut(idx)?),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected dictionary")),
        }
    }
}
