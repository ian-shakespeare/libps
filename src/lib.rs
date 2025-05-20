#![feature(allocator_api)]

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
    execution_stack: Vec<Object>,
    operand_stack: Vec<Object>,
    stdout: StdoutLock<'static>,
}

impl Interpreter {
    pub fn new(stdout: StdoutLock<'static>) -> Self {
        let mut system_dict = DictionaryObject::default();
        system_dict.insert(
            Object::Name("cvlit".into()),
            Object::Operator((OperatorObject::Cvlit, Mode::Executable)),
        );
        system_dict.insert(
            Object::Name("cvx".into()),
            Object::Operator((OperatorObject::Cvx, Mode::Executable)),
        );
        system_dict.insert(
            Object::Name("exec".into()),
            Object::Operator((OperatorObject::Exec, Mode::Executable)),
        );
        system_dict.insert(
            Object::Name("flush".into()),
            Object::Operator((OperatorObject::Flush, Mode::Executable)),
        );
        system_dict.insert(
            Object::Name("print".into()),
            Object::Operator((OperatorObject::Print, Mode::Executable)),
        );
        system_dict.insert(
            Object::Name("quit".into()),
            Object::Operator((OperatorObject::Quit, Mode::Executable)),
        );

        let dict_stack = vec![Rc::new(RefCell::new(system_dict))];

        Self {
            execution_stack: Vec::new(),
            operand_stack: Vec::new(),
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
            Object::Operator((operator, _)) => {
                match operator {
                    OperatorObject::Cvlit => self.cvlit(),
                    OperatorObject::Cvx => self.cvx(),
                    OperatorObject::Exec => self.exec(),
                    OperatorObject::Flush => self.flush(),
                    OperatorObject::Print => self.print(),
                    OperatorObject::Quit => process::exit(0),
                }
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

    fn initiate_error(&mut self, _source: Object, error: Error) {
        // TODO: actually initiate the error
        println!("{error}");
    }

    fn into_value<T>(refcounter: Rc<RefCell<T>>) -> crate::Result<T> {
        let value = Rc::try_unwrap(refcounter)
            .or(Err(Error::new(
                ErrorKind::Unregistered,
                "failed to dereference composite",
            )))?
            .into_inner();

        Ok(value)
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
        self.operand_stack
            .pop()
            .ok_or(Error::from(ErrorKind::StackUnderflow))
    }

    fn pop_string(&mut self) -> crate::Result<Rc<RefCell<StringObject>>> {
        match self.pop()? {
            Object::String(string) => Ok(string),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected string")),
        }
    }
}

// Builtins
impl Interpreter {
    /// *any* __cvlit__ *any*
    ///
    /// (convert to literal) makes the object on the top of the operand stack have the liter-
    /// al instead of the executable attribute.
    ///
    /// Errors: __stackunderflow__
    /// See Also: __cvx__, __xcheck__
    pub fn cvlit(&mut self) -> crate::Result<()> {
        let mut obj = self.pop()?;

        if obj.mode().is_executable() {
            obj.set_mode(Mode::Literal);
        }

        self.operand_stack.push(obj);

        Ok(())
    }

    /// *any* __cvx__ *any*
    ///
    /// (convert to executable) makes the object on the top of the operand stack have the
    /// executable instead of the literal attribute.
    ///
    /// Errors: __stackunderflow__
    /// See Also: __cvlit__, __xcheck__
    pub fn cvx(&mut self) -> crate::Result<()> {
        let mut obj = self.pop()?;

        if obj.mode().is_literal() {
            obj.set_mode(Mode::Executable);
        }

        self.operand_stack.push(obj);

        Ok(())
    }

    /// *any* __exec__ -
    ///
    /// pushes the operand on the execution stack, executing it immediately. The effect of
    /// executing an object depends on the object’s type and literal/executable attribute;
    /// see Section 3.5, “Execution.” In particular, executing a literal object will cause it
    /// only to be pushed back on the operand stack. Executing a procedure, however,
    /// will cause the procedure to be called.
    ///
    /// # Examples
    /// ```postscript
    /// (3 2 add) cvx exec ⇒ 5
    /// 3 2 /add exec ⇒ 3 2 /add
    /// 3 2 /add cvx exec ⇒ 5
    /// ```
    ///
    /// In the first example, the string 3 2 add is made executable and then executed. Exe-
    /// cuting a string causes its characters to be scanned and interpreted according to the
    /// PostScript language syntax rules.
    ///
    /// In the second example, the literal objects 3, 2, and /add are pushed on the operand
    /// stack, then __exec__ is applied to /add. Since /add is a literal name, executing it simply
    /// causes it to be pushed back on the operand stack. The __exec__ operator in this case
    /// has no useful effect.
    ///
    /// In the third example, the literal name /add on the top of the operand stack is
    /// made executable by __cvx__. Applying __exec__ to this executable name causes it to be
    /// looked up and the add operation to be performed.
    ///
    /// Errors: __stackunderflow__
    /// See Also: __xcheck__, __cvx__, __run__
    pub fn exec(&mut self) -> crate::Result<()> {
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
                self.initiate_error(
                    Object::Operator((OperatorObject::Exec, Mode::Executable)),
                    e,
                );
            },
        };

        Ok(())
    }

    /// – __flush__ –
    ///
    /// causes any buffered characters for the standard output file to be delivered imme-
    /// diately. In general, a program requiring output to be sent immediately, such as
    /// during real-time, two-way interactions, should call __flush__ after generating that out-
    /// put.
    ///
    /// Errors: __ioerror__
    /// See Also: __flushfile__, __print__
    pub fn flush(&mut self) -> crate::Result<()> {
        self.stdout.flush().or(Err(Error::from(ErrorKind::IoError)))
    }

    /// *string* __print__ –
    ///
    /// writes the characters of string to the standard output file (see Section 3.8, “File In-
    /// put and Output”). This operator provides the simplest means of sending text to
    /// an application or an interactive user. Note that __print__ is a file operator; it has noth-
    /// ing to do with painting glyphs for characters on the current page (see __show__) or
    /// with sending the current page to a raster output device (see __showpage__).
    ///
    /// Errors: __invalidaccess__, __ioerror__, __stackunderflow__, __typecheck__
    /// See Also: __write__, __flush__, __=__, __==__, __printobject__
    pub fn print(&mut self) -> crate::Result<()> {
        let string = Self::into_value(self.pop_string()?)?;

        if !string.access().is_readable() {
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
