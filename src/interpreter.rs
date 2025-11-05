use std::{
    io::{BufRead, Cursor, StdinLock, StdoutLock, Write},
    marker::PhantomData,
};

use crate::{
    array::ArrayObject,
    composite::{Address, Composite, CompositeValue, Container},
    dictionary::DictionaryObject,
    name::NameObject,
    object::Access,
    operator::{Operator, OperatorObject},
    scanner::{Scanner, Token},
    string::StringObject,
    Error, ErrorKind, FileObject, Mode, Object,
};

pub struct Global;
pub struct Local;

pub struct Interpreter<MemoryRegion = Global> {
    global_memory: Container<CompositeValue>,
    local_memory: Container<CompositeValue>,
    operand_stack: Vec<Object>,
    execution_stack: Vec<Object>,
    dictionary_stack: Vec<DictionaryObject>,
    stdin: StdinLock<'static>,
    stdout: StdoutLock<'static>,

    memory_region: PhantomData<MemoryRegion>,
}

impl Interpreter<Global> {
    fn create_composite<T: Into<CompositeValue>>(&mut self, value: T) -> Composite {
        let address = self.global_memory.insert(value.into());

        Composite {
            address,
            access: Access::default(),
            mode: Mode::default(),
        }
    }

    fn get_composite_value<'a, T>(&'a self, address: Address) -> crate::Result<&'a T>
    where
        &'a T: TryFrom<&'a CompositeValue>,
    {
        let composite = self
            .global_memory
            .get(address)
            .ok_or(Error::from(ErrorKind::Undefined))?;

        composite
            .try_into()
            .map_err(|_| Error::from(ErrorKind::TypeCheck))
    }
}

impl Interpreter<Local> {
    fn create_composite<T: Into<CompositeValue>>(&mut self, value: T) -> Composite {
        let address = self.local_memory.insert(value.into());

        Composite {
            address,
            access: Access::default(),
            mode: Mode::default(),
        }
    }

    fn get_composite_value<'a, T>(&'a self, address: Address) -> crate::Result<&'a T>
    where
        &'a T: TryFrom<&'a CompositeValue>,
    {
        let composite = self
            .local_memory
            .get(address)
            .or_else(|| self.global_memory.get(address))
            .ok_or(Error::from(ErrorKind::Undefined))?;

        composite
            .try_into()
            .map_err(|_| Error::from(ErrorKind::TypeCheck))
    }
}

impl Interpreter {
    fn execute_stack(&mut self) -> crate::Result<()> {
        let obj = self
            .execution_stack
            .pop()
            .ok_or(Error::from(ErrorKind::StackUnderflow))?;

        match obj {
            Object::Name(name) => {
                let def = self.find(&Object::Name(name))?;
                self.operand_stack.push(def.clone());
                self.exec()
            },
            Object::Operator(obj) => {
                match obj.operator() {
                    Operator::Cvlit => self.cvlit(),
                    Operator::Cvx => self.cvx(),
                    Operator::Exec => self.exec(),
                    Operator::Flush => self.flush(),
                    Operator::Print => self.print(),
                    Operator::Quit => todo!(),
                }
            },
            Object::Array(obj) => {
                let array = self
                    .get_composite_value::<ArrayObject>(obj.address)
                    .cloned()?;

                for obj in array.iter() {
                    self.operand_stack.push(obj.clone()); // TODO: find a way to avoid cloning here
                    self.exec()?;
                }

                Ok(())
            },
            Object::File(obj) => {
                let file = self
                    .get_composite_value::<FileObject>(obj.address)
                    .cloned()?;
                let scanner: Scanner<FileObject> = Scanner::new(file);

                self.execute_tokens(scanner)
            },
            Object::String(obj) => {
                let string = self
                    .get_composite_value::<StringObject>(obj.address)
                    .cloned()?;
                let cursor = Cursor::new(string.value());
                let scanner: Scanner<Cursor<&[u8]>> = Scanner::new(cursor);

                self.execute_tokens(scanner)
            },
            _ => Err(Error::new(ErrorKind::Unregistered, "not implemented")),
        }
    }

    fn execute_tokens<I: Iterator<Item = crate::Result<Token>>>(
        &mut self,
        tokens: I,
    ) -> crate::Result<()> {
        for token in tokens {
            let obj = match token? {
                Token::Integer(value) => Ok(Object::Integer(value)),
                Token::Real(value) => Ok(Object::Real(value)),
                Token::String(value) => {
                    let composite = self.create_composite(StringObject::from(value));
                    Ok(Object::String(composite))
                },
                Token::Name(value) => Ok(Object::Name(NameObject::from(value))),
                Token::LiteralName(value) => {
                    let mut name = NameObject::from(value);
                    name.mode = Mode::Literal;

                    Ok(Object::Name(name))
                },
                Token::Procedure(_) => todo!(),
            };

            self.operand_stack.push(obj?);
            self.exec()?;
        }

        Ok(())
    }

    fn initiate_error(&mut self, _source: Object, error: Error) {
        // TODO: actually initiate the error
        println!("{error}");
    }

    fn find(&self, key: &Object) -> crate::Result<Object> {
        for dict in self.dictionary_stack.iter().rev() {
            if let Some(obj) = dict.get(key).cloned() {
                return Ok(obj);
            }
        }

        println!("find error");
        Err(Error::from(ErrorKind::Undefined))
    }

    fn push(&mut self, object: Object) {
        self.operand_stack.push(object);
    }

    fn pop(&mut self) -> crate::Result<Object> {
        self.operand_stack
            .pop()
            .ok_or(Error::from(ErrorKind::StackUnderflow))
    }

    fn pop_comp(&mut self) -> crate::Result<Composite> {
        match self.pop()? {
            Object::Array(comp)
            | Object::Dictionary(comp)
            | Object::File(comp)
            | Object::GState(comp)
            | Object::PackedArray(comp)
            | Object::Save(comp)
            | Object::String(comp) => Ok(comp),
            _ => Err(Error::from(ErrorKind::TypeCheck)),
        }
    }
}

impl Interpreter<Global> {
    pub fn new(stdin: StdinLock<'static>, stdout: StdoutLock<'static>) -> Self {
        let mut global_memory: Container<CompositeValue> = Container::default();

        let ops = vec![
            ("exec", Operator::Exec),
            ("flush", Operator::Flush),
            ("print", Operator::Print),
            ("quit", Operator::Quit),
        ];

        let mut system_dict = DictionaryObject::default();
        for (key, value) in ops {
            let address = global_memory.insert(CompositeValue::String(StringObject::from(key)));
            let key = Composite {
                address,
                access: Access::ExecuteOnly,
                mode: Mode::Executable,
            };

            let value = OperatorObject::from(value);
            system_dict.insert(Object::String(key), Object::Operator(value));
        }

        let dictionary_stack = vec![system_dict];

        Self {
            dictionary_stack,
            global_memory,
            stdin,
            stdout,
            execution_stack: Vec::new(),
            local_memory: Container::default(),
            operand_stack: Vec::new(),

            memory_region: PhantomData,
        }
    }
}

impl Interpreter {
    pub fn cvlit(&mut self) -> crate::Result<()> {
        let mut obj = self.pop()?;

        if obj.mode().is_executable() {
            obj.set_mode(Mode::Literal);
        }

        self.operand_stack.push(obj);

        Ok(())
    }

    pub fn cvx(&mut self) -> crate::Result<()> {
        let mut obj = self.pop()?;

        if obj.mode().is_literal() {
            obj.set_mode(Mode::Executable);
        }

        self.operand_stack.push(obj);

        Ok(())
    }

    pub fn exec(&mut self) -> crate::Result<()> {
        println!("EXECING");
        match self.pop() {
            Ok(obj) => {
                match obj.mode() {
                    Mode::Literal => {
                        self.operand_stack.push(obj);
                    },
                    Mode::Executable => {
                        self.execution_stack.push(obj.clone());

                        match self.execute_stack() {
                            Ok(_) => (),
                            Err(e) => self.initiate_error(obj, e),
                        }
                    },
                }
            },
            Err(e) => {
                let op = OperatorObject::from(Operator::Exec);
                self.initiate_error(Object::Operator(op), e);
            },
        };

        Ok(())
    }

    pub fn executive(&mut self) -> crate::Result<()> {
        let _ = write!(self.stdout, "libPS version 0.0.0\n"); // TODO: parameterize this version
        let _ = self.stdout.flush();

        loop {
            let _ = write!(self.stdout, "PS>");
            let _ = self.stdout.flush();

            let mut input = String::new();
            self.stdin
                .read_line(&mut input)
                .or(Err(Error::from(ErrorKind::IoError)))?;

            if input.trim() == "quit" {
                break;
            }

            let composite = self.create_composite(StringObject::from(input));
            self.operand_stack.push(Object::String(composite));
            self.exec()?;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> crate::Result<()> {
        self.stdout.flush().or(Err(Error::from(ErrorKind::IoError)))
    }

    pub fn print(&mut self) -> crate::Result<()> {
        println!("PRINTING");
        let composite = self.pop_comp()?;
        let string = self
            .get_composite_value::<StringObject>(composite.address)
            .cloned()?;

        if !composite.access.is_readable() {
            return Err(Error::from(ErrorKind::InvalidAccess));
        }

        // TODO: check stdout for write permissions

        let _ = self
            .stdout
            .write(string.value())
            .or(Err(Error::from(ErrorKind::IoError)))?;

        Ok(())
    }
}
