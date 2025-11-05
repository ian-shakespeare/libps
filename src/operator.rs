use std::{fmt, hash::Hash};

use crate::Mode;

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum Operator {
    /// *any* __cvlit__ *any*
    ///
    /// (convert to literal) makes the object on the top of the operand stack have the liter-
    /// al instead of the executable attribute.
    ///
    /// Errors: __stackunderflow__
    /// See Also: __cvx__, __xcheck__
    Cvlit,

    /// *any* __cvx__ *any*
    ///
    /// (convert to executable) makes the object on the top of the operand stack have the
    /// executable instead of the literal attribute.
    ///
    /// Errors: __stackunderflow__
    /// See Also: __cvlit__, __xcheck__
    Cvx,

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
    Exec,

    /// – __flush__ –
    ///
    /// causes any buffered characters for the standard output file to be delivered imme-
    /// diately. In general, a program requiring output to be sent immediately, such as
    /// during real-time, two-way interactions, should call __flush__ after generating that out-
    /// put.
    ///
    /// Errors: __ioerror__
    /// See Also: __flushfile__, __print__
    Flush,

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
    Print,
    Quit,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Cvlit => "cvlit".fmt(f),
            Operator::Cvx => "cvx".fmt(f),
            Operator::Exec => "exec".fmt(f),
            Operator::Flush => "flush".fmt(f),
            Operator::Print => "print".fmt(f),
            Operator::Quit => "quit".fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct OperatorObject {
    operator: Operator,
    pub mode: Mode,
}

impl OperatorObject {
    pub fn operator(&self) -> Operator {
        self.operator
    }
}

impl From<Operator> for OperatorObject {
    fn from(value: Operator) -> Self {
        Self {
            operator: value,
            mode: Mode::Executable,
        }
    }
}

impl Hash for OperatorObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.operator.hash(state)
    }
}

impl PartialEq for OperatorObject {
    fn eq(&self, other: &Self) -> bool {
        self.operator == other.operator
    }
}
