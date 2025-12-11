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
                self.exec();

                Ok(())
            },
            Object::Null(mode) => {
                if mode.is_literal() {
                    self.operand_stack.push(Object::Null(Mode::Literal));
                }

                Ok(())
            },
            Object::Operator(obj) => {
                match obj.operator() {
                    Operator::Cvlit => self.cvlit(),
                    Operator::Cvx => self.cvx(),
                    Operator::Exec => self.exec(),
                    Operator::Flush => self.flush(),
                    Operator::Print => self.print(),
                    Operator::Quit => todo!(),
                    Operator::Undefined => println!("received an `undefined` error"),
                };

                Ok(())
            },
            Object::Array(obj) => {
                let array = self
                    .get_composite_value::<ArrayObject>(obj.address)
                    .cloned()?;

                for obj in array.iter() {
                    self.operand_stack.push(obj.clone()); // TODO: find a way to avoid cloning here
                    self.exec();
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
                match obj.mode {
                    Mode::Literal => {
                        self.operand_stack.push(Object::String(obj));

                        Ok(())
                    },
                    Mode::Executable => {
                        let string = self
                            .get_composite_value::<StringObject>(obj.address)
                            .cloned()?;
                        let cursor = Cursor::new(string.value());
                        let scanner: Scanner<Cursor<&[u8]>> = Scanner::new(cursor);

                        self.execute_tokens(scanner)
                    },
                }
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
                    let mut composite = self.create_composite(StringObject::from(value));
                    composite.mode = Mode::Literal;

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
            self.exec();
        }

        Ok(())
    }

    fn find(&self, key: &Object) -> crate::Result<Object> {
        for dict in self.dictionary_stack.iter().rev() {
            if let Some(obj) = dict.get(key).cloned() {
                return Ok(obj);
            }
        }

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
        let mut local_memory: Container<CompositeValue> = Container::default();

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

        let mut error_info = DictionaryObject::default();
        error_info.insert(
            Object::Name(NameObject::from("newerror")),
            Object::Boolean(false),
        );
        error_info.insert(
            Object::Name(NameObject::from("errorinfo")),
            Object::Null(Mode::Executable),
        );
        error_info.insert(
            Object::Name(NameObject::from("recordstacks")),
            Object::Boolean(true),
        );
        error_info.insert(
            Object::Name(NameObject::from("binary")),
            Object::Boolean(false),
        );

        let error_info_addr = local_memory.insert(CompositeValue::Dictionary(error_info));
        system_dict.insert(
            Object::Name(NameObject::from("$error")),
            Object::Dictionary(Composite {
                address: error_info_addr,
                access: Access::Unlimited,
                mode: Mode::Executable,
            }),
        );

        let mut error_dict = DictionaryObject::default();
        error_dict.insert(
            Object::Name(NameObject::from("undefined")),
            Object::Operator(OperatorObject::from(Operator::Undefined)),
        );

        let error_dict_addr = local_memory.insert(CompositeValue::Dictionary(error_dict));
        system_dict.insert(
            Object::Name(NameObject::from("errordict")),
            Object::Dictionary(Composite {
                address: error_dict_addr,
                access: Access::Unlimited,
                mode: Mode::Executable,
            }),
        );

        let dictionary_stack = vec![system_dict];

        Self {
            dictionary_stack,
            global_memory,
            local_memory,
            stdin,
            stdout,
            execution_stack: Vec::new(),
            operand_stack: Vec::new(),

            memory_region: PhantomData,
        }
    }

    fn operate<F: Fn(&mut Self) -> crate::Result<()>>(&mut self, action: F) {
        let backup = self.operand_stack.clone();

        if let Err(e) = action(self) {
            self.operand_stack = backup;

            // TODO: push cause object to operand stack

            let error_dict_addr = self
                .find(&Object::Name(NameObject::from("errordict")))
                .expect("failed to find errordict in dict stack")
                .into_composite()
                .expect("errordict must be composite")
                .address;

            let error_dict: &DictionaryObject = self
                .local_memory
                .get(error_dict_addr)
                .expect("failed to find errordict in global mem")
                .try_into()
                .expect("errordict must be a dictionary");

            let kind = e.kind();
            let error_obj: Object = e.into();
            let handler = error_dict
                .get(&error_obj)
                .cloned()
                .expect(&format!("failed to find handler for '{:?}'", kind));

            self.execution_stack.push(handler);
            self.execute_stack()
                .expect("failed to execute error handler");
        }
    }
}

impl Interpreter {
    pub fn cvlit(&mut self) {
        self.operate(|ctx| {
            let mut obj = ctx.pop()?;

            if obj.mode().is_executable() {
                obj.set_mode(Mode::Literal);
            }

            ctx.operand_stack.push(obj);

            Ok(())
        });
    }

    pub fn cvx(&mut self) {
        self.operate(|ctx| {
            let mut obj = ctx.pop()?;

            if obj.mode().is_literal() {
                obj.set_mode(Mode::Executable);
            }

            ctx.operand_stack.push(obj);

            Ok(())
        });
    }

    pub fn exec(&mut self) {
        self.operate(|ctx| {
            let obj = ctx.pop()?;
            match obj.mode() {
                Mode::Literal => {
                    ctx.operand_stack.push(obj);
                },
                Mode::Executable => {
                    ctx.execution_stack.push(obj.clone());
                    ctx.execute_stack()?;
                },
            };

            Ok(())
        });
    }

    pub fn executive(&mut self) {
        self.operate(|ctx| {
            let _ = write!(ctx.stdout, "libPS version 0.0.0\n"); // TODO: parameterize this version
            let _ = ctx.stdout.flush();

            loop {
                let _ = write!(ctx.stdout, "PS>");
                let _ = ctx.stdout.flush();

                let mut input = String::new();
                ctx.stdin
                    .read_line(&mut input)
                    .or(Err(Error::from(ErrorKind::IoError)))?;

                if input.trim() == "quit" {
                    break;
                }

                let composite = ctx.create_composite(StringObject::from(input));
                ctx.operand_stack.push(Object::String(composite));
                ctx.exec();
            }

            Ok(())
        });
    }

    pub fn flush(&mut self) {
        self.operate(|ctx| ctx.stdout.flush().or(Err(Error::from(ErrorKind::IoError))));
    }

    pub fn print(&mut self) {
        self.operate(|ctx| {
            let composite = ctx.pop_comp()?;
            let string = ctx
                .get_composite_value::<StringObject>(composite.address)
                .cloned()?;

            if !composite.access.is_readable() {
                return Err(Error::from(ErrorKind::InvalidAccess));
            }

            // TODO: check stdout for write permissions

            let _ = ctx
                .stdout
                .write(string.value())
                .or(Err(Error::from(ErrorKind::IoError)))?;

            Ok(())
        });
    }
}
