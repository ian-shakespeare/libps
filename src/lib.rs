use std::{
    cell::RefCell,
    io::{StdoutLock, Write},
    process,
    rc::Rc,
    result,
};

use dictionary::DictionaryObject;
pub use error::{Error, ErrorKind};
pub use file::FileObject;
use lexer::Lexer;
pub use object::{Mode, Object};
use operator::OperatorObject;
use string::StringObject;

mod array;
mod dictionary;
mod encoding;
mod error;
mod file;
mod lexer;
mod name;
mod object;
mod operator;
mod string;

type Result<T> = result::Result<T, Error>;

pub struct Interpreter {
    dict_stack: Vec<Rc<RefCell<DictionaryObject>>>,
    operand_stack: Vec<Object>,
    stdout: StdoutLock<'static>,
    transaction_stack: Vec<Object>,
}

impl Interpreter {
    pub fn new(stdout: StdoutLock<'static>) -> Self {
        let mut system_dict = DictionaryObject::default();
        system_dict.insert(
            Object::String(Rc::new(RefCell::new("flush".into()))),
            Object::Operator((OperatorObject::Flush, Mode::Executable)),
        );
        system_dict.insert(
            Object::String(Rc::new(RefCell::new("print".into()))),
            Object::Operator((OperatorObject::Print, Mode::Executable)),
        );
        system_dict.insert(
            Object::String(Rc::new(RefCell::new("quit".into()))),
            Object::Operator((OperatorObject::Quit, Mode::Executable)),
        );

        let dict_stack = vec![Rc::new(RefCell::new(system_dict))];

        Self {
            operand_stack: Vec::new(),
            transaction_stack: Vec::new(),
            dict_stack,
            stdout,
        }
    }

    pub fn push_file(&mut self, file: FileObject) {
        self.operand_stack
            .push(Object::File(Rc::new(RefCell::new(file))));
    }

    pub fn push_string(&mut self, string: StringObject) {
        self.operand_stack
            .push(Object::String(Rc::new(RefCell::new(string))))
    }

    fn execute_object(&mut self, obj: Object) -> crate::Result<()> {
        match obj {
            Object::Name(name) => {
                let def = self.find(&Object::Name(name))?;
                self.operand_stack.push(def.clone());
                self.exec()
            },
            Object::Operator((operator, _)) => match operator {
                OperatorObject::Flush => self.flush(),
                OperatorObject::Print => self.print(),
                OperatorObject::Quit => process::exit(0),
            },
            Object::Array(array) => {
                let array = array.borrow();

                for obj in array.iter() {
                    self.operand_stack.push(obj.clone());
                    self.exec()?;
                }

                Ok(())
            },
            Object::File(file) => {
                let lexer = Lexer::new(file.borrow().clone());

                for obj in lexer {
                    self.operand_stack.push(obj?);
                    self.exec()?;
                }

                Ok(())
            },
            Object::String(string) => {
                let value: Vec<u8> = string.borrow().value().into();
                let lexer = Lexer::new(FileObject::from(value));

                for obj in lexer {
                    self.operand_stack.push(obj?);
                    self.exec()?;
                }

                Ok(())
            },
            _ => Err(Error::new(ErrorKind::Unregistered, "not implemented")),
        }
    }

    fn clear_transaction(&mut self) {
        self.transaction_stack.clear();
    }

    fn initiate_error(&mut self, e: Error) {
        // TODO: actually initiate the error
        println!("{e}");
    }

    fn find(&self, key: &Object) -> crate::Result<Object> {
        for dict in self.dict_stack.iter().rev() {
            if let Some(obj) = dict.borrow().get(key).cloned() {
                return Ok(obj);
            }
        }

        Err(Error::new(
            ErrorKind::Undefined,
            format!("`{key}` not found"),
        ))
    }

    fn pop(&mut self) -> crate::Result<Object> {
        let obj = self
            .operand_stack
            .pop()
            .ok_or(Error::from(ErrorKind::StackUnderflow))?;
        self.transaction_stack.push(obj.clone());

        Ok(obj)
    }

    fn pop_string(&mut self) -> crate::Result<Rc<RefCell<StringObject>>> {
        match self.pop()? {
            Object::String(s) => Ok(s),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }
}

impl Interpreter {
    pub fn exec(&mut self) -> crate::Result<()> {
        match self.pop() {
            Ok(obj) => match obj.mode() {
                Mode::Literal => {
                    self.operand_stack.push(obj);
                    self.clear_transaction();
                },
                Mode::Executable => match self.execute_object(obj) {
                    Ok(_) => self.clear_transaction(),
                    Err(e) => self.initiate_error(e),
                },
            },
            Err(e) => {
                self.initiate_error(e);
            },
        };

        Ok(())
    }

    pub fn flush(&mut self) -> crate::Result<()> {
        self.stdout.flush().or(Err(Error::from(ErrorKind::IoError)))
    }

    pub fn print(&mut self) -> crate::Result<()> {
        let s = self.pop_string()?;

        let _ = self
            .stdout
            .write(s.borrow().value())
            .or(Err(Error::from(ErrorKind::IoError)))?;

        Ok(())
    }
}
