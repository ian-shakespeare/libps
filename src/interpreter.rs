use std::io;

use crate::{
    object::{Access, Container, Object, PostScriptArray, PostScriptDictionary, PostScriptString},
    operators,
    rand::RandomNumberGenerator,
    Error, ErrorKind, Lexer,
};

#[allow(dead_code)]
pub struct InterpreterState {
    pub arrays: Container<PostScriptArray>,
    pub dict_stack: Vec<usize>,
    pub dicts: Container<PostScriptDictionary>,
    pub execution_stack: Vec<Object>,
    pub is_packing: bool,
    pub operand_stack: Vec<Object>,
    pub rng: RandomNumberGenerator,
    pub strings: Container<PostScriptString>,
}

impl Default for InterpreterState {
    fn default() -> Self {
        let mut dicts = Container::default();

        let mut system_dict = PostScriptDictionary::from(operators::system_dict());
        system_dict.set_access(Access::ExecuteOnly);
        let system_dict_idx = dicts.insert(system_dict);

        // TODO: Insert global & user dict

        Self {
            arrays: Container::default(),
            dict_stack: vec![system_dict_idx],
            execution_stack: Vec::default(),
            is_packing: false,
            operand_stack: Vec::default(),
            rng: RandomNumberGenerator::default(),
            strings: Container::default(),
            dicts,
        }
    }
}

impl InterpreterState {
    pub fn push(&mut self, obj: Object) {
        self.operand_stack.push(obj);
    }

    pub fn pop(&mut self) -> crate::Result<Object> {
        match self.operand_stack.pop() {
            Some(obj) => Ok(obj),
            None => Err(Error::from(ErrorKind::StackUnderflow)),
        }
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
}

#[derive(Default)]
#[repr(transparent)]
pub struct Interpreter {
    state: InterpreterState,
}

impl Interpreter {
    pub fn evaluate<I>(&mut self, mut lexer: Lexer<I>) -> crate::Result<()>
    where
        I: Iterator<Item = char>,
    {
        while let Some(obj) = lexer.next_obj(&mut self.state.strings, &mut self.state.arrays) {
            self.execute_object(obj?)?;
        }
        Ok(())
    }

    fn execute_object(&mut self, obj: Object) -> crate::Result<()> {
        match obj {
            Object::Integer(_)
            | Object::Real(_)
            | Object::Boolean(_)
            | Object::String(_)
            | Object::Procedure(_)
            | Object::Literal(_) => {
                self.state.push(obj);
                Ok(())
            },
            Object::Operator(op) => op(&mut self.state),
            Object::Name(name) => {
                for dict_idx in self.state.dict_stack.iter().rev() {
                    if let Ok(dict) = self.state.dicts.get(*dict_idx) {
                        if let Ok(obj) = dict.get(name.clone()) {
                            return self.execute_object(obj.clone());
                        }
                    }
                }

                Err(Error::new(ErrorKind::Undefined, name))
            },
            _ => Err(Error::new(ErrorKind::Unregistered, "not implemented")),
        }
    }

    pub fn write_stack(&self, writer: &mut impl io::Write) -> io::Result<usize> {
        let mut count = 0;

        for obj in &self.state.operand_stack {
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
                    self.state
                        .strings
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
                    .state
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
                    .state
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
            Object::Name(name) | Object::Literal(name) => writer.write(name.as_bytes()),
            Object::Mark => writer.write(b"mark"),
            Object::Null => writer.write(b"null"),
            _ => Ok(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    const ERROR_MARGIN: f64 = 0.000005;

    #[test]
    fn test_dup() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 dup".chars().into())?;

        assert_eq!(2, interpreter.state.operand_stack.len());
        assert_eq!(
            Some(Object::Integer(1)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(1)),
            interpreter.state.operand_stack.pop()
        );

        Ok(())
    }

    #[test]
    fn test_exch() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 exch".chars().into())?;

        assert_eq!(
            Some(Object::Integer(1)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            interpreter.state.operand_stack.pop()
        );

        Ok(())
    }

    #[test]
    fn test_exch_underflow() {
        let cases = ["1 exch", "exch"];

        for input in cases {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_pop() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 pop".chars().into())?;

        assert_eq!(1, interpreter.state.operand_stack.len());
        assert_eq!(
            Some(Object::Integer(1)),
            interpreter.state.operand_stack.pop()
        );

        Ok(())
    }

    #[test]
    fn test_pop_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("pop".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_simple() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 3 2 copy".chars().into())?;

        // Stack should be: 3 2 3 2 1 |
        assert_eq!(5, interpreter.state.operand_stack.len());
        assert_eq!(
            Some(Object::Integer(3)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(3)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(1)),
            interpreter.state.operand_stack.pop()
        );

        Ok(())
    }

    #[test]
    fn test_copy_simple_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("copy".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_composite() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 ] [ 4 5 6 ] copy".chars().into())?;

        // Stack should be: [ 1 2 3 ] |
        assert_eq!(1, interpreter.state.operand_stack.len());
        let Some(Object::Array(arr_idx)) = interpreter.state.operand_stack.pop() else {
            return Err("expected array object".into());
        };

        let arr = interpreter.state.arrays.get(arr_idx)?;

        assert_eq!(Some(Object::Integer(1)), arr.value().get(0).cloned());
        assert_eq!(Some(Object::Integer(2)), arr.value().get(1).cloned());
        assert_eq!(Some(Object::Integer(3)), arr.value().get(2).cloned());

        Ok(())
    }

    #[test]
    fn test_copy_composite_rangecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("[ 1 2 3 ] [ 4 5 ] copy".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_composite_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 [ 1 2 3 ] copy".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_composite_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("[ 1 2 3 ] copy".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_roll() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 3 3 1 roll".chars().into())?;

        assert_eq!(3, interpreter.state.operand_stack.len());
        assert_eq!(
            Some(Object::Integer(2)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(1)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(3)),
            interpreter.state.operand_stack.pop()
        );

        interpreter.state.operand_stack.clear();
        interpreter.evaluate("1 2 3 3 -1 roll".chars().into())?;

        assert_eq!(3, interpreter.state.operand_stack.len());
        assert_eq!(
            Some(Object::Integer(1)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(3)),
            interpreter.state.operand_stack.pop()
        );
        assert_eq!(
            Some(Object::Integer(2)),
            interpreter.state.operand_stack.pop()
        );

        Ok(())
    }

    #[test]
    fn test_roll_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("roll".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_index() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 2 3 0 index", 3),
            ("1 2 3 1 index", 2),
            ("1 2 3 2 index", 1),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.state.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_index_rangecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 2 3 4 index".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_index_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("index".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_mark() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("mark".chars().into())?;

        assert!(matches!(
            interpreter.state.operand_stack.pop(),
            Some(Object::Mark)
        ));

        Ok(())
    }

    #[test]
    fn test_clear() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 3 clear".chars().into())?;

        assert_eq!(0, interpreter.state.operand_stack.len());

        Ok(())
    }

    #[test]
    fn test_count() -> Result<(), Box<dyn error::Error>> {
        let cases = [("count", 0), ("1 2 3 count", 3)];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.state.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_counttomark() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 mark 2 3 counttomark".chars().into())?;

        assert_eq!(
            Some(Object::Integer(2)),
            interpreter.state.operand_stack.pop()
        );

        Ok(())
    }

    #[test]
    fn test_counttomark_unmatchedmark() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 2 3 counttomark 3".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::UnmatchedMark, result.unwrap_err().kind());
    }

    #[test]
    fn test_cleartomark() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 mark 2 3 cleartomark".chars().into())?;

        assert_eq!(1, interpreter.state.operand_stack.len());
        assert_eq!(
            Some(Object::Integer(1)),
            interpreter.state.operand_stack.pop()
        );

        Ok(())
    }

    #[test]
    fn test_add() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 1 add", Object::Integer(2)),
            ("1 1.5 add", Object::Real(2.5)),
            ("1.5 1 add", Object::Real(2.5)),
            ("1.5 1.5 add", Object::Real(3.0)),
            ("16#7FFFFFFF 1 add", Object::Real(f64::from(i32::MAX) + 1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(Some(expect), interpreter.state.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_add_typecheck() {
        let cases = ["(string) 1.2e7 add", "1.2e7 (string) add"];

        for input in cases {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_add_underflow() {
        let cases = ["1 add", "add"];

        for input in cases {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_div() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("2 2 div", 1.0),
            ("1 3 div", 0.333333),
            ("-1 3 div", -0.333333),
            ("1 0.5 div", 2.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            let Some(Object::Real(value)) = interpreter.state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_div_undefinedresult() {
        let cases = ["1 0 div", "1 0.0 div"];

        for input in cases {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::UndefinedResult, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_idiv() -> Result<(), Box<dyn error::Error>> {
        let cases = [("1 2 idiv", 0), ("10 3 idiv", 3), ("5 2 idiv", 2)];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.state.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_idiv_undefinedresult() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 0 idiv".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::UndefinedResult, result.unwrap_err().kind());
    }

    #[test]
    fn test_imod() -> Result<(), Box<dyn error::Error>> {
        let cases = [("13 2 imod", 1), ("16 3 imod", 1)];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.state.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_imod_undefinedresult() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("2 0 imod".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::UndefinedResult, result.unwrap_err().kind());
    }

    #[test]
    fn test_mul() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("2 2 mul", Object::Integer(4)),
            ("2 2.5 mul", Object::Real(5.0)),
            ("2.5 2 mul", Object::Real(5.0)),
            ("2.5 2.5 mul", Object::Real(6.25)),
            ("16#7FFFFFFF 2 mul", Object::Real(f64::from(i32::MAX) * 2.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(Some(expect), interpreter.state.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_sub() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("2 2 sub", Object::Integer(0)),
            ("2 2.5 sub", Object::Real(-0.5)),
            ("2.5 2 sub", Object::Real(0.5)),
            ("2.5 2.5 sub", Object::Real(0.0)),
            ("-2147483648 1 sub", Object::Real(f64::from(i32::MIN) - 1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(Some(expect), interpreter.state.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_abs() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 abs", Object::Integer(1)),
            ("-1 abs", Object::Integer(1)),
            ("16#7FFFFFFF abs", Object::Integer(i32::MAX)),
            ("-2147483648 abs", Object::Real(-1.0 * f64::from(i32::MIN))),
            ("1.0 abs", Object::Real(1.0)),
            ("-1.0 abs", Object::Real(1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(Some(expect), interpreter.state.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_neg() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("-1 neg", Object::Integer(1)),
            ("1 neg", Object::Integer(-1)),
            ("-1.0 neg", Object::Real(1.0)),
            ("1.0 neg", Object::Real(-1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(Some(expect), interpreter.state.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_ceiling() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 ceiling", Object::Integer(1)),
            ("1.0 ceiling", Object::Real(1.0)),
            ("1.0000001 ceiling", Object::Real(2.0)),
            ("1.1 ceiling", Object::Real(2.0)),
            ("1.2 ceiling", Object::Real(2.0)),
            ("1.3 ceiling", Object::Real(2.0)),
            ("1.4 ceiling", Object::Real(2.0)),
            ("1.5 ceiling", Object::Real(2.0)),
            ("1.6 ceiling", Object::Real(2.0)),
            ("1.7 ceiling", Object::Real(2.0)),
            ("1.8 ceiling", Object::Real(2.0)),
            ("1.9 ceiling", Object::Real(2.0)),
            ("1.9999999 ceiling", Object::Real(2.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(Some(expect), interpreter.state.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_floor() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 floor", Object::Integer(1)),
            ("1.0 floor", Object::Real(1.0)),
            ("1.0000001 floor", Object::Real(1.0)),
            ("1.1 floor", Object::Real(1.0)),
            ("1.2 floor", Object::Real(1.0)),
            ("1.3 floor", Object::Real(1.0)),
            ("1.4 floor", Object::Real(1.0)),
            ("1.5 floor", Object::Real(1.0)),
            ("1.6 floor", Object::Real(1.0)),
            ("1.7 floor", Object::Real(1.0)),
            ("1.8 floor", Object::Real(1.0)),
            ("1.9 floor", Object::Real(1.0)),
            ("1.9999999 floor", Object::Real(1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(Some(expect), interpreter.state.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_round() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 round", Object::Integer(1)),
            ("1.0 round", Object::Real(1.0)),
            ("1.0000001 round", Object::Real(1.0)),
            ("1.1 round", Object::Real(1.0)),
            ("1.2 round", Object::Real(1.0)),
            ("1.3 round", Object::Real(1.0)),
            ("1.4 round", Object::Real(1.0)),
            ("1.4999999 round", Object::Real(1.0)),
            ("1.5 round", Object::Real(2.0)),
            ("1.6 round", Object::Real(2.0)),
            ("1.7 round", Object::Real(2.0)),
            ("1.8 round", Object::Real(2.0)),
            ("1.9 round", Object::Real(2.0)),
            ("1.9999999 round", Object::Real(2.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(Some(expect), interpreter.state.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_truncate() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 truncate", Object::Integer(1)),
            ("1.0 truncate", Object::Real(1.0)),
            ("1.0000001 truncate", Object::Real(1.0)),
            ("1.1 truncate", Object::Real(1.0)),
            ("1.2 truncate", Object::Real(1.0)),
            ("1.3 truncate", Object::Real(1.0)),
            ("1.4 truncate", Object::Real(1.0)),
            ("1.5 truncate", Object::Real(1.0)),
            ("1.6 truncate", Object::Real(1.0)),
            ("1.7 truncate", Object::Real(1.0)),
            ("1.8 truncate", Object::Real(1.0)),
            ("1.9 truncate", Object::Real(1.0)),
            ("1.9999999 truncate", Object::Real(1.0)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(Some(expect), interpreter.state.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_sqrt() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 sqrt", 1.0),
            ("1.0 sqrt", 1.0),
            ("2 sqrt", 1.414213),
            ("2.0 sqrt", 1.414213),
            ("4 sqrt", 2.0),
            ("4.0 sqrt", 2.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());

            let received = interpreter.state.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_atan() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("0 1 atan", 0.0),
            ("0.0 1.0 atan", 0.0),
            ("1 0 atan", 90.0),
            ("1.0 0.0 atan", 90.0),
            ("-100 0 atan", 270.0),
            ("-100.0 0.0 atan", 270.0),
            ("4 4 atan", 45.0),
            ("4.0 4.0 atan", 45.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert_eq!(
                Some(Object::Real(expect)),
                interpreter.state.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_cos() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("0 cos", 1.0),
            ("0.0 cos", 1.0),
            ("90 cos", 0.0),
            ("-90.0 cos", 0.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());

            let received = interpreter.state.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_sin() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("0 sin", 0.0),
            ("0.0 sin", 0.0),
            ("90 sin", 1.0),
            ("-90.0 sin", -1.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());

            let received = interpreter.state.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_exp() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("1 100 exp", 1.0),
            ("1.0 100.0 exp", 1.0),
            ("2 8 exp", 256.0),
            ("2.0 8.0 exp", 256.0),
            ("8 0.33333333 exp", 2.0),
            ("8.0 0.33333333 exp", 2.0),
            ("9 0.5 exp", 3.0),
            ("9.0 0.5 exp", 3.0),
            ("-9 -1 exp", -0.111111),
            ("-9.0 -1.0 exp", -0.111111),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());

            let received = interpreter.state.pop_real()?;
            assert!((received - expect) < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_ln() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("10 ln", 2.302585),
            ("10.0 ln", 2.302585),
            ("100 ln", 4.60517),
            ("100.0 ln", 4.60517),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());

            let received = interpreter.state.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_log() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("10 log", 1.0),
            ("10.0 log", 1.0),
            ("100 log", 2.0),
            ("100.0 log", 2.0),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());

            let received = interpreter.state.pop_real()?;
            assert!((received - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_rand() -> Result<(), Box<dyn error::Error>> {
        for _ in 0..5 {
            let mut interpreter = Interpreter::default();
            interpreter.evaluate("rand".chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            assert!(interpreter.state.pop_int()? >= 0);
        }

        Ok(())
    }

    #[test]
    fn test_srand() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("100 srand rand".chars().into())?;
        let initial_value = interpreter.state.pop_int()?;

        interpreter.state.operand_stack.clear();
        interpreter.evaluate("1 srand rand".chars().into())?;
        let intermediate_value = interpreter.state.pop_int()?;
        assert_ne!(initial_value, intermediate_value);

        interpreter.state.operand_stack.clear();
        interpreter.evaluate("100 srand rand".chars().into())?;
        let end_value = interpreter.state.pop_int()?;
        assert_eq!(initial_value, end_value);

        Ok(())
    }

    #[test]
    fn test_rrand() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 srand rrand".chars().into())?;

        assert_eq!(1, interpreter.state.pop_int()?);

        Ok(())
    }

    #[test]
    fn test_array() -> Result<(), Box<dyn error::Error>> {
        for num in 0..5 {
            let input = format!("{num} array");
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            let Some(Object::Array(arr_idx)) = interpreter.state.operand_stack.pop() else {
                return Err("expected array object".into());
            };

            let arr = interpreter.state.arrays.get(arr_idx)?;
            assert_eq!(num, arr.len());
            assert!(arr.value().iter().all(|obj| matches!(obj, Object::Null)));
        }

        Ok(())
    }

    #[test]
    fn test_array_rangecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("-1 array".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind())
    }

    #[test]
    fn test_array_typeheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("(str) array".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind())
    }

    #[test]
    fn test_array_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("array".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind())
    }

    #[test]
    fn test_startarray() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[".chars().into())?;

        assert_eq!(1, interpreter.state.operand_stack.len());

        let obj = interpreter
            .state
            .operand_stack
            .pop()
            .ok_or("expected object")?;
        assert!(matches!(obj, Object::Mark));

        Ok(())
    }

    #[test]
    fn test_endarray() -> Result<(), Box<dyn error::Error>> {
        for length in 0..5 {
            let input = "[ ".to_string() + &format!("0 ").repeat(length) + "]";

            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            let Some(Object::Array(arr_idx)) = interpreter.state.operand_stack.pop() else {
                return Err("expected array object".into());
            };

            let arr = interpreter.state.arrays.get(arr_idx)?;
            assert_eq!(length, arr.len());
            assert!(arr
                .value()
                .iter()
                .all(|obj| matches!(obj, Object::Integer(0))));
        }

        Ok(())
    }

    #[test]
    fn test_endarray_unmatchedmark() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("]".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::UnmatchedMark, result.unwrap_err().kind());
    }

    #[test]
    fn test_length() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 4 5 ] length".chars().into())?;

        assert_eq!(1, interpreter.state.operand_stack.len());
        assert_eq!(5, interpreter.state.pop_int()?);

        Ok(())
    }

    #[test]
    fn test_length_packedarray() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("5 packedarray length".chars().into())?;

        assert_eq!(1, interpreter.state.operand_stack.len());
        assert_eq!(5, interpreter.state.pop_int()?);

        Ok(())
    }

    #[test]
    fn test_get() -> Result<(), Box<dyn error::Error>> {
        for i in 0..5 {
            let input = format!("[ 1 2 3 4 5 ] {i} get");
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            let received = interpreter.state.pop_int()?;

            assert_eq!(i + 1, received);
        }

        Ok(())
    }

    #[test]
    fn test_get_packedarray() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 packedarray 0 get".chars().into())?;

        assert_eq!(1, interpreter.state.operand_stack.len());
        assert!(matches!(interpreter.state.pop()?, Object::Null));

        Ok(())
    }

    #[test]
    fn test_get_rangecheck() {
        let inputs = ["[ 1 2 3 ] -1 get", "[ 1 2 3 ] 3 get", "[ ] 0 get"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_get_typecheck() {
        let inputs = ["[ 1 2 3 ] (str) get", "1 3 get"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_get_underflow() {
        let inputs = ["0 get", "get"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_put() -> Result<(), Box<dyn error::Error>> {
        for i in 0..5 {
            let input = format!("5 array {i} 3.14 put");
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(0, interpreter.state.operand_stack.len());

            let arr = interpreter.state.arrays.get(1)?;
            assert_eq!(Some(Object::Real(3.14)), arr.value().get(i).cloned());
        }

        Ok(())
    }

    #[test]
    fn test_put_rangecheck() {
        let inputs = [
            "[ 1 2 3 ] -1 3.14 put",
            "[ 1 2 3 ] 3 3.14 put",
            "[ ] 0 3.14 put",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_put_typecheck() {
        let inputs = ["[ 1 2 3 ] (str) 3.14 put", "0 3 3.14 put"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_put_underflow() {
        let inputs = ["0 3.14 put", "3.14 put", "put"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_getinterval() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 4 5 ] 1 3 getinterval".chars().into())?;

        assert_eq!(1, interpreter.state.operand_stack.len());

        let arr = interpreter.state.pop_array()?;
        assert_eq!(
            vec![Object::Integer(2), Object::Integer(3), Object::Integer(4)],
            arr.value().clone()
        );

        Ok(())
    }

    #[test]
    fn test_getinterval_rangecheck() {
        let inputs = [
            "[ 1 2 3 ] -1 0 getinterval",
            "[ 1 2 3 ] 0 -1 getinterval",
            "[ 1 2 3 ] 1000 0 getinterval",
            "[ 1 2 3 ] 0 1000 getinterval",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_getinterval_typecheck() {
        let inputs = [
            "3.14 0 1 getinterval",
            "[ 1 ] (str) 1 getinterval",
            "[ 1 ] 0 (str) getinterval",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_getinterval_underflow() {
        let inputs = ["0 1 getinterval", "1 getinterval", "getinterval"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_putinterval() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 4 5 ] 1 [ 6 7 8 ] putinterval".chars().into())?;

        let arr = interpreter.state.arrays.get(1)?;
        assert_eq!(
            vec![
                Object::Integer(1),
                Object::Integer(6),
                Object::Integer(7),
                Object::Integer(8),
                Object::Integer(5),
            ],
            arr.value().clone()
        );

        Ok(())
    }

    #[test]
    fn test_putinterval_rangecheck() {
        let inputs = [
            "[ 1 2 3 ] 1000 [ 4 ] putinterval",
            "[ 1 2 3 ] -1 [ 4 ] putinterval",
            "[ 1 2 3 ] 2 [ 4 5 ] putinterval",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_putinterval_typecheck() {
        let inputs = [
            "3.14 0 [ 4 ] putinterval",
            "[ 1 2 3 ] (str) [ 4 ] putinterval",
            "[ 1 2 3 ] 0 3.14 putinterval",
        ];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_putinterval_underflow() {
        let inputs = ["0 [ 4 ] putinterval", "[ 4 ] putinterval", "putinterval"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_astore() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("1 2 3 4 5 3 array astore".chars().into())?;

        assert_eq!(2, interpreter.state.operand_stack.len());

        let arr = interpreter.state.arrays.get(1)?;
        assert_eq!(
            vec![Object::Integer(3), Object::Integer(4), Object::Integer(5)],
            arr.value().clone()
        );

        Ok(())
    }

    #[test]
    fn test_astore_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 2 3 3.14 astore".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_astore_underflow() {
        let inputs = ["1 2 3 10 array astore", "astore"];

        for input in inputs {
            let mut interpreter = Interpreter::default();
            let result = interpreter.evaluate(input.chars().into());

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_aload() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("[ 1 2 3 4 5 ] aload".chars().into())?;

        assert_eq!(6, interpreter.state.operand_stack.len());

        let arr = interpreter.state.pop_array()?;
        assert_eq!(
            vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
                Object::Integer(5)
            ],
            arr.value().clone()
        );

        assert_eq!(5, interpreter.state.pop_int()?);
        assert_eq!(4, interpreter.state.pop_int()?);
        assert_eq!(3, interpreter.state.pop_int()?);
        assert_eq!(2, interpreter.state.pop_int()?);
        assert_eq!(1, interpreter.state.pop_int()?);

        Ok(())
    }

    #[test]
    fn test_aload_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("1 aload".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_aload_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("aload".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_packedarray() -> Result<(), Box<dyn error::Error>> {
        for num in 0..5 {
            let input = format!("{num} packedarray");
            let mut interpreter = Interpreter::default();
            interpreter.evaluate(input.chars().into())?;

            assert_eq!(1, interpreter.state.operand_stack.len());
            let Some(Object::PackedArray(arr_idx)) = interpreter.state.operand_stack.pop() else {
                return Err("expected packed array object".into());
            };

            let arr = interpreter.state.arrays.get(arr_idx)?;
            assert!(arr.access().is_read_only());
            assert_eq!(num, arr.len());
            assert!(arr.value().iter().all(|obj| matches!(obj, Object::Null)));
        }

        Ok(())
    }

    #[test]
    fn test_packedarray_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("(str) packedarray".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_packedarray_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("packedarray".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_setpacking() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("true setpacking".chars().into())?;

        assert!(interpreter.state.is_packing);

        Ok(())
    }

    #[test]
    fn test_setpacking_typecheck() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("(str) setpacking".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_setpacking_underflow() {
        let mut interpreter = Interpreter::default();
        let result = interpreter.evaluate("setpacking".chars().into());

        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_currentpacking() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::default();
        interpreter.evaluate("true setpacking currentpacking".chars().into())?;

        assert_eq!(1, interpreter.state.operand_stack.len());
        assert!(interpreter.state.pop_bool()?);

        Ok(())
    }
}
