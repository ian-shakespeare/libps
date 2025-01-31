use std::{collections, f64::consts};

use crate::{
    object::{Access, Composite, Container, Object},
    rand::RandomNumberGenerator,
    Error, ErrorKind, Lexer,
};

pub struct Interpreter<I: Iterator<Item = char>> {
    lexer: Lexer<I>,
    operand_stack: Vec<Object>,
    execution_stack: Vec<Object>,
    global_dict: collections::HashMap<String, Object>,
    user_dict: collections::HashMap<String, Object>,
    strings: Container<Composite<String>>,
    arrays: Container<Composite<Vec<Object>>>,
    rng: RandomNumberGenerator,
}

impl<I> From<Lexer<I>> for Interpreter<I>
where
    I: Iterator<Item = char>,
{
    fn from(value: Lexer<I>) -> Self {
        Self {
            lexer: value,
            operand_stack: Vec::new(),
            execution_stack: Vec::new(),
            global_dict: collections::HashMap::new(),
            user_dict: collections::HashMap::new(),
            strings: Container::new(),
            arrays: Container::new(),
            rng: RandomNumberGenerator::default(),
        }
    }
}

impl<I> Interpreter<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(input: I) -> Self {
        Self {
            lexer: Lexer::from(input),
            operand_stack: Vec::new(),
            execution_stack: Vec::new(),
            global_dict: collections::HashMap::new(),
            user_dict: collections::HashMap::new(),
            strings: Container::new(),
            arrays: Container::new(),
            rng: RandomNumberGenerator::default(),
        }
    }

    pub fn evaluate(&mut self) -> crate::Result<()> {
        while let Some(obj) = self.lexer.next_obj(&mut self.strings) {
            self.execute_object(obj?)?;
        }
        Ok(())
    }

    fn execute_object(&mut self, obj: Object) -> crate::Result<()> {
        match obj {
            Object::Integer(_) | Object::Real(_) | Object::Boolean(_) => {
                self.operand_stack.push(obj)
            }
            Object::Name(name) => {
                if let Some(obj) = self.user_dict.get(&name) {
                    return self.execute_object(obj.clone());
                }

                if let Some(obj) = self.global_dict.get(&name) {
                    return self.execute_object(obj.clone());
                }

                // system_dict
                match name.as_str() {
                    "dup" => self.dup()?,
                    "exch" => self.exch()?,
                    "pop" => {
                        self.pop()?;
                    }
                    "copy" => self.copy()?,
                    "roll" => self.roll()?,
                    "index" => self.index()?,
                    "mark" => self.mark()?,
                    "clear" => self.clear()?,
                    "count" => self.count()?,
                    "counttomark" => self.counttomark()?,
                    "cleartomark" => self.cleartomark()?,
                    "add" => self.arithmetic(i32::checked_add, |a: f64, b: f64| a + b)?,
                    "div" => self.arithmetic(i32::checked_div, |a: f64, b: f64| a / b)?,
                    "idiv" => todo!(),
                    "imod" => todo!(),
                    "mul" => self.arithmetic(i32::checked_mul, |a: f64, b: f64| a * b)?,
                    "sub" => self.arithmetic(i32::checked_sub, |a: f64, b: f64| a - b)?,
                    "abs" => self.num_unary(i32::checked_abs, f64::abs)?,
                    "neg" => self.num_unary(i32::checked_neg, |a: f64| -1.0 * a)?,
                    "ceiling" => self.num_unary(|a: i32| Some(a), f64::ceil)?,
                    "floor" => self.num_unary(|a: i32| Some(a), f64::floor)?,
                    "round" => self.num_unary(|a: i32| Some(a), f64::round)?,
                    "truncate" => self.num_unary(|a: i32| Some(a), f64::trunc)?,
                    "sqrt" => self.num_unary(|_| None, f64::sqrt)?,
                    "atan" => self.arithmetic(
                        |_, _| None,
                        |den: f64, num: f64| {
                            positive_degrees(radians_to_degrees(
                                (f64::from(num) / f64::from(den)).atan(),
                            ))
                        },
                    )?,
                    "cos" => self.num_unary(|_| None, f64::cos)?,
                    "sin" => self.num_unary(|_| None, f64::sin)?,
                    "exp" => self.arithmetic(|_, _| None, |exp: f64, base: f64| base.powf(exp))?,
                    "ln" => self.num_unary(|_| None, f64::ln)?,
                    "log" => self.num_unary(|_| None, f64::log10)?,
                    "rand" => self.rand()?,
                    "srand" => self.srand()?,
                    "rrand" => self.rrand()?,
                    "array" => self.array()?,
                    "[" => self.start_array()?,
                    "]" => self.end_array()?,
                    "length" => self.length()?,
                    "get" => self.get()?,
                    "put" => self.put()?,
                    "getinterval" => self.getinterval()?,
                    "putinterval" => self.putinterval()?,
                    "astore" => self.astore()?,
                    "aload" => self.aload()?,
                    _ => return Err(Error::new(ErrorKind::Undefined, format!("{name}"))),
                }
            }
            _ => {}
        }

        Ok(())
    }
}

// Operators
impl<I> Interpreter<I>
where
    I: Iterator<Item = char>,
{
    fn dup(&mut self) -> crate::Result<()> {
        let obj = self.pop()?;

        self.push(obj.clone());
        self.push(obj);

        Ok(())
    }

    fn exch(&mut self) -> crate::Result<()> {
        let first = self.pop()?;
        let second = self.pop()?;

        self.push(first);
        self.push(second);

        Ok(())
    }

    fn copy(&mut self) -> crate::Result<()> {
        let mut original_stack = Vec::new();
        let mut copy_stack = Vec::new();

        match self.pop()? {
            Object::Integer(n) => {
                for _ in 0..n {
                    let obj = self.pop()?;
                    copy_stack.push(obj.clone());
                    original_stack.push(obj);
                }

                while let Some(obj) = original_stack.pop() {
                    self.push(obj);
                }

                while let Some(obj) = copy_stack.pop() {
                    self.push(obj);
                }

                Ok(())
            }
            Object::Array(dest_idx) => {
                let source = self.pop_array()?;

                if source.is_exec_only() {
                    return Err(Error::from(ErrorKind::InvalidAccess));
                }

                let source = source.inner.clone();

                let destination = self.arrays.get_mut(dest_idx)?;

                if destination.is_read_only() {
                    return Err(Error::from(ErrorKind::InvalidAccess));
                }

                for (index, obj) in source.into_iter().enumerate() {
                    match destination.inner.get_mut(index) {
                        Some(dest_obj) => *dest_obj = obj,
                        None => return Err(Error::from(ErrorKind::RangeCheck)),
                    }
                }

                self.push(Object::Array(dest_idx));

                Ok(())
            }
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        }
    }

    fn roll(&mut self) -> crate::Result<()> {
        let mut temp_stack = Vec::new();
        let mut shift_stack = Vec::new();

        let j = self.pop_int()?;
        let n = self.pop_int()?;

        let j = j % n;
        for i in 0..n {
            let obj = self.pop()?;
            if i < j {
                shift_stack.push(obj);
            } else {
                temp_stack.push(obj);
            }
        }

        while let Some(obj) = shift_stack.pop() {
            self.push(obj);
        }

        while let Some(obj) = temp_stack.pop() {
            self.push(obj);
        }

        Ok(())
    }

    fn index(&mut self) -> crate::Result<()> {
        let idx = self.pop_usize()?;

        if self.operand_stack.len() == 0 {
            return Err(Error::from(ErrorKind::RangeCheck));
        }

        let Some(idx) = (self.operand_stack.len() - 1).checked_sub(idx) else {
            return Err(Error::from(ErrorKind::RangeCheck));
        };

        match self.operand_stack.get(idx) {
            Some(obj) => {
                self.push(obj.clone());
                Ok(())
            }
            None => Err(Error::from(ErrorKind::RangeCheck)),
        }
    }

    fn mark(&mut self) -> crate::Result<()> {
        self.push(Object::Mark);

        Ok(())
    }

    fn clear(&mut self) -> crate::Result<()> {
        self.operand_stack.clear();

        Ok(())
    }

    fn count(&mut self) -> crate::Result<()> {
        let i = usize_to_i32(self.operand_stack.len())?;

        self.push(Object::Integer(i));

        Ok(())
    }

    fn counttomark(&mut self) -> crate::Result<()> {
        let Some((u, _)) = self
            .operand_stack
            .iter()
            .rev()
            .enumerate()
            .find(|(_, obj)| matches!(obj, Object::Mark))
        else {
            return Err(Error::from(ErrorKind::UnmatchedMark));
        };

        let i = usize_to_i32(u)?;

        self.push(Object::Integer(i));

        Ok(())
    }

    fn cleartomark(&mut self) -> crate::Result<()> {
        loop {
            match self.pop() {
                Err(_) => return Err(Error::from(ErrorKind::UnmatchedMark)),
                Ok(obj) => {
                    if matches!(obj, Object::Mark) {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn arithmetic(
        &mut self,
        checked: impl Fn(i32, i32) -> Option<i32>,
        real: impl Fn(f64, f64) -> f64,
    ) -> crate::Result<()> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;

        if lhs.is_int() && rhs.is_int() {
            let obj = match checked(lhs.into_int()?, rhs.into_int()?) {
                Some(total) => Object::Integer(total),
                None => Object::Real(real(lhs.into_real()?, rhs.into_real()?)),
            };

            self.push(obj);

            return Ok(());
        }

        let total = real(lhs.into_real()?, rhs.into_real()?);

        if !total.is_finite() || total.is_nan() {
            return Err(Error::from(ErrorKind::UndefinedResult));
        }

        self.push(Object::Real(total));

        Ok(())
    }

    fn num_unary(
        &mut self,
        int: impl Fn(i32) -> Option<i32>,
        real: impl Fn(f64) -> f64,
    ) -> crate::Result<()> {
        let n = self.pop()?;

        if n.is_int() {
            let obj = match int(n.into_int()?) {
                Some(total) => Object::Integer(total),
                None => Object::Real(real(n.into_real()?)),
            };

            self.push(obj);

            return Ok(());
        }

        self.push(Object::Real(real(n.into_real()?)));

        Ok(())
    }

    fn rand(&mut self) -> crate::Result<()> {
        let n = self.rng.rand();

        self.push(Object::Integer(n));

        Ok(())
    }

    fn srand(&mut self) -> crate::Result<()> {
        let seed = self.pop_int()?;

        self.rng.set_seed(seed);

        Ok(())
    }

    fn rrand(&mut self) -> crate::Result<()> {
        let seed = self.rng.seed();

        self.push(Object::Integer(seed));

        Ok(())
    }

    fn array(&mut self) -> crate::Result<()> {
        let len = self.pop_usize()?;

        let inner = vec![Object::Null; len];
        let composite = Composite {
            access: Access::default(),
            len,
            inner,
        };

        let idx = self.arrays.insert(composite);
        self.push(Object::Array(idx));

        Ok(())
    }

    fn start_array(&mut self) -> crate::Result<()> {
        self.push(Object::Mark);

        Ok(())
    }

    fn end_array(&mut self) -> crate::Result<()> {
        let mut arr = Vec::new();

        loop {
            match self.pop() {
                Err(_) => return Err(Error::from(ErrorKind::UnmatchedMark)),
                Ok(obj) => {
                    if matches!(obj, Object::Mark) {
                        break;
                    }

                    arr.push(obj);
                }
            }
        }

        arr.reverse();

        let len = arr.len();
        let composite = Composite {
            access: Access::default(),
            inner: arr,
            len,
        };

        let idx = self.arrays.insert(composite);
        self.push(Object::Array(idx));

        Ok(())
    }

    fn length(&mut self) -> crate::Result<()> {
        let arr = self.pop_array()?;

        if arr.is_exec_only() {
            return Err(Error::from(ErrorKind::InvalidAccess));
        }

        let i = usize_to_i32(arr.len)?;
        self.push(Object::Integer(i));

        Ok(())
    }

    fn get(&mut self) -> crate::Result<()> {
        let index = self.pop_usize()?;
        let arr = self.pop_array()?;

        if arr.is_exec_only() {
            return Err(Error::from(ErrorKind::InvalidAccess));
        }

        let Some(obj) = arr.inner.get(index).cloned() else {
            return Err(Error::from(ErrorKind::RangeCheck));
        };

        self.push(obj);

        Ok(())
    }

    fn put(&mut self) -> crate::Result<()> {
        let value = self.pop()?;
        let index = self.pop_usize()?;
        let arr = self.pop_array_mut()?;

        if arr.is_read_only() || arr.is_exec_only() {
            return Err(Error::from(ErrorKind::InvalidAccess));
        }

        let Some(obj) = arr.inner.get_mut(index) else {
            return Err(Error::from(ErrorKind::RangeCheck));
        };

        *obj = value;

        Ok(())
    }

    fn getinterval(&mut self) -> crate::Result<()> {
        let count = self.pop_usize()?;
        let index = self.pop_usize()?;
        let arr = self.pop_array()?;

        if arr.is_exec_only() {
            return Err(Error::from(ErrorKind::InvalidAccess));
        }

        if index >= arr.len {
            return Err(Error::from(ErrorKind::RangeCheck));
        }

        let mut subarr = Vec::with_capacity(count);

        for i in index..(index + count) {
            let Some(obj) = arr.inner.get(i).cloned() else {
                return Err(Error::from(ErrorKind::RangeCheck));
            };

            subarr.push(obj);
        }

        let composite = Composite {
            access: Access::default(),
            inner: subarr,
            len: count,
        };

        let idx = self.arrays.insert(composite);
        self.push(Object::Array(idx));

        Ok(())
    }

    fn putinterval(&mut self) -> crate::Result<()> {
        let source = self.pop_array()?;

        if source.is_exec_only() {
            return Err(Error::from(ErrorKind::InvalidAccess));
        }

        let source = source.inner.clone();

        let index = self.pop_usize()?;
        let destination = self.pop_array_mut()?;

        if destination.is_read_only() || destination.is_exec_only() {
            return Err(Error::from(ErrorKind::InvalidAccess));
        }

        for (offset, obj) in source.into_iter().enumerate() {
            let Some(dest_obj) = destination.inner.get_mut(index + offset) else {
                return Err(Error::from(ErrorKind::RangeCheck));
            };

            *dest_obj = obj;
        }

        Ok(())
    }

    fn astore(&mut self) -> crate::Result<()> {
        let Object::Array(arr_idx) = self.pop()? else {
            return Err(Error::new(ErrorKind::TypeCheck, "expected array"));
        };

        let len = match self.arrays.get(arr_idx) {
            Ok(Composite { len, .. }) => Ok(len.clone()),
            Err(_) => Err(Error::from(ErrorKind::Undefined)),
        }?;

        let mut stored = Vec::with_capacity(len);

        for _ in 0..len {
            stored.push(self.pop()?);
        }

        match self.arrays.get_mut(arr_idx) {
            Ok(composite) => {
                composite.inner = stored;
                Ok(())
            }
            Err(_) => Err(Error::from(ErrorKind::Undefined)),
        }
    }

    fn aload(&mut self) -> crate::Result<()> {
        let arr = self.pop_array()?;

        if arr.is_exec_only() {
            return Err(Error::from(ErrorKind::InvalidAccess));
        }

        let arr = arr.inner.clone();

        for obj in arr {
            self.push(obj.clone());
        }

        Ok(())
    }
}

// Utils
impl<I> Interpreter<I>
where
    I: Iterator<Item = char>,
{
    fn push(&mut self, obj: Object) {
        self.operand_stack.push(obj);
    }

    fn pop(&mut self) -> crate::Result<Object> {
        match self.operand_stack.pop() {
            Some(obj) => Ok(obj),
            None => Err(Error::from(ErrorKind::StackUnderflow)),
        }
    }

    fn pop_int(&mut self) -> crate::Result<i32> {
        match self.pop()? {
            Object::Integer(i) => Ok(i),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected integer")),
        }
    }

    fn pop_real(&mut self) -> crate::Result<f64> {
        match self.pop()? {
            Object::Real(r) => Ok(r),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected real")),
        }
    }

    fn pop_usize(&mut self) -> crate::Result<usize> {
        let i = self.pop_int()?;

        match i.try_into() {
            Ok(u) => Ok(u),
            Err(_) => Err(Error::from(ErrorKind::RangeCheck)),
        }
    }

    fn pop_array(&mut self) -> crate::Result<&Composite<Vec<Object>>> {
        match self.pop()? {
            Object::Array(idx) => Ok(self.arrays.get(idx)?),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }

    fn pop_array_mut(&mut self) -> crate::Result<&mut Composite<Vec<Object>>> {
        match self.pop()? {
            Object::Array(idx) => Ok(self.arrays.get_mut(idx)?),
            _ => Err(Error::new(ErrorKind::TypeCheck, "expected array")),
        }
    }
}

fn radians_to_degrees(radians: f64) -> f64 {
    radians * (180.0 / consts::PI)
}

fn degrees_to_radians(degrees: f64) -> f64 {
    (degrees * consts::PI) / 180.0
}

fn positive_degrees(degrees: f64) -> f64 {
    if degrees < 0.0 {
        360.0 + degrees
    } else {
        degrees
    }
}

fn usize_to_i32(u: usize) -> crate::Result<i32> {
    let i: i32 = match u.try_into() {
        Ok(i) => Ok(i),
        Err(_) => Err(Error::new(
            ErrorKind::Unregistered,
            "failed to convert usize to int",
        )),
    }?;

    Ok(i)
}

#[cfg(test)]
mod tests {
    use std::error;

    use crate::interpreter;

    use super::*;

    const ERROR_MARGIN: f64 = 0.000005;

    #[test]
    fn test_dup() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("1 dup".chars());

        interpreter.evaluate()?;

        assert_eq!(2, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_exch() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("1 2 exch".chars());

        interpreter.evaluate()?;

        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_exch_underflow() {
        let cases = ["1 exch", "exch"];

        for input in cases {
            let mut interpreter = Interpreter::new(input.chars());

            let result = interpreter.evaluate();

            assert!(result.is_err());
            assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_pop() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("1 2 pop".chars());

        interpreter.evaluate()?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_pop_underflow() {
        let mut interpreter = Interpreter::new("pop".chars());

        let result = interpreter.evaluate();
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_simple() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("1 2 3 2 copy".chars());
        interpreter.evaluate()?;

        // Stack should be: 3 2 3 2 1 |
        assert_eq!(5, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(3)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(3)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_copy_simple_underflow() {
        let mut interpreter = Interpreter::new("copy".chars());

        let result = interpreter.evaluate();
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_composite() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("[ 1 2 3 ] [ 4 5 6 ] copy".chars());

        interpreter.evaluate()?;

        // Stack should be: [ 1 2 3 ] |
        assert_eq!(1, interpreter.operand_stack.len());
        let Some(Object::Array(arr_idx)) = interpreter.operand_stack.pop() else {
            return Err("expected array object".into());
        };

        let arr = interpreter.arrays.get(arr_idx)?;

        assert_eq!(Some(Object::Integer(1)), arr.inner.get(0).cloned());
        assert_eq!(Some(Object::Integer(2)), arr.inner.get(1).cloned());
        assert_eq!(Some(Object::Integer(3)), arr.inner.get(2).cloned());

        Ok(())
    }

    #[test]
    fn test_copy_composite_rangecheck() {
        let mut interpreter = Interpreter::new("[ 1 2 3 ] [ 4 5 ] copy".chars());

        let result = interpreter.evaluate();
        assert!(result.is_err());
        assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_composite_typecheck() {
        let mut interpreter = Interpreter::new("1 [ 1 2 3 ] copy".chars());

        let result = interpreter.evaluate();
        assert!(result.is_err());
        assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_copy_composite_underflow() {
        let mut interpreter = Interpreter::new("[ 1 2 3 ] copy".chars());

        let result = interpreter.evaluate();
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_roll() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("1 2 3 3 2 roll".chars());

        interpreter.evaluate()?;

        assert_eq!(3, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(3)), interpreter.operand_stack.pop());
        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_roll_underflow() {
        let mut interpreter = Interpreter::new("roll".chars());

        let result = interpreter.evaluate();
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_index_rangecheck() {
        let mut interpreter = Interpreter::new("1 2 3 4 index".chars());

        let result = interpreter.evaluate();
        assert!(result.is_err());
        assert_eq!(ErrorKind::RangeCheck, result.unwrap_err().kind());
    }

    #[test]
    fn test_index_underflow() {
        let mut interpreter = Interpreter::new("index".chars());

        let result = interpreter.evaluate();
        assert!(result.is_err());
        assert_eq!(ErrorKind::StackUnderflow, result.unwrap_err().kind());
    }

    #[test]
    fn test_mark() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("mark".chars());

        interpreter.evaluate()?;

        assert!(matches!(
            interpreter.operand_stack.pop(),
            Some(Object::Mark)
        ));

        Ok(())
    }

    #[test]
    fn test_clear() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("1 2 3 clear".chars());

        interpreter.evaluate()?;

        assert_eq!(0, interpreter.operand_stack.len());

        Ok(())
    }

    #[test]
    fn test_count() -> Result<(), Box<dyn error::Error>> {
        let cases = [("count", 0), ("1 2 3 count", 3)];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_counttomark() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("1 mark 2 3 counttomark".chars());

        interpreter.evaluate()?;

        assert_eq!(Some(Object::Integer(2)), interpreter.operand_stack.pop());

        Ok(())
    }

    #[test]
    fn test_counttomark_unmatchedmark() {
        let mut interpreter = Interpreter::new("1 2 3 counttomark 3".chars());

        let result = interpreter.evaluate();
        assert!(result.is_err());
        assert_eq!(ErrorKind::UnmatchedMark, result.unwrap_err().kind());
    }

    #[test]
    fn test_cleartomark() -> Result<(), Box<dyn error::Error>> {
        let mut interpreter = Interpreter::new("1 mark 2 3 cleartomark".chars());

        interpreter.evaluate()?;

        assert_eq!(1, interpreter.operand_stack.len());
        assert_eq!(Some(Object::Integer(1)), interpreter.operand_stack.pop());

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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
        }

        Ok(())
    }

    #[test]
    fn test_add_typecheck() {
        let cases = ["(string) 1.2e7 add", "1.2e7 (string) add"];

        for input in cases {
            let mut interpreter = Interpreter::new(input.chars());

            let result = interpreter.evaluate();
            assert!(result.is_err());
            assert_eq!(ErrorKind::TypeCheck, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_add_underflow() {
        let cases = ["1 add", "add"];

        for input in cases {
            let mut interpreter = Interpreter::new(input.chars());

            let result = interpreter.evaluate();
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            let Some(Object::Real(value)) = interpreter.operand_stack.pop() else {
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
            let mut interpreter = Interpreter::new(input.chars());

            let result = interpreter.evaluate();
            assert!(result.is_err());
            assert_eq!(ErrorKind::UndefinedResult, result.unwrap_err().kind());
        }
    }

    #[test]
    fn test_idiv() -> Result<(), Box<dyn error::Error>> {
        let cases = [("1 2 idiv", 0), ("10 3 idiv", 3), ("5 2 idiv", 2)];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_idiv_undefinedresult() {
        let mut interpreter = Interpreter::new("1 0 idiv".chars());

        let result = interpreter.evaluate();
        assert!(result.is_err());
        assert_eq!(ErrorKind::UndefinedResult, result.unwrap_err().kind());
    }

    #[test]
    fn test_imod() -> Result<(), Box<dyn error::Error>> {
        let cases = [("13 2 imod", 1), ("16 3 imod", 1)];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(
                Some(Object::Integer(expect)),
                interpreter.operand_stack.pop()
            );
        }

        Ok(())
    }

    #[test]
    fn test_imod_undefinedresult() {
        let mut interpreter = Interpreter::new("2 0 imod".chars());

        let result = interpreter.evaluate();
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
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
            ("1.7976931348623157e308 abs", Object::Real(f64::MAX)),
            ("-1.7976931348623157e308 abs", Object::Real(f64::MAX)),
        ];

        for (input, expect) in cases {
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            assert_eq!(Some(expect), interpreter.operand_stack.pop());
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
            let mut interpreter = Interpreter::new(input.chars());

            interpreter.evaluate()?;

            assert_eq!(1, interpreter.operand_stack.len());
            let Some(Object::Real(value)) = interpreter.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_atan() {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(0), Object::Integer(1), 0.0),
            (Object::Real(0.0), Object::Real(1.0), 0.0),
            (Object::Integer(1), Object::Integer(0), 90.0),
            (Object::Real(1.0), Object::Real(0.0), 90.0),
            (Object::Integer(-100), Object::Integer(0), 270.0),
            (Object::Real(-100.0), Object::Real(0.0), 270.0),
            (Object::Integer(4), Object::Integer(4), 45.0),
            (Object::Real(4.0), Object::Real(4.0), 45.0),
        ];

        for (num, den, expect) in cases {
            execution_state.operand_stack.push(num);
            execution_state.operand_stack.push(den);

            assert!(atan(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            assert_eq!(
                Some(Object::Real(expect)),
                execution_state.operand_stack.pop()
            );
        }
    }

    #[test]
    fn test_cos() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(0), 1.0),
            (Object::Real(0.0), 1.0),
            (Object::Integer(90), 0.0),
            (Object::Real(-90.0), 0.0),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(cos(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_sin() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(0), 0.0),
            (Object::Real(0.0), 0.0),
            (Object::Integer(90), 1.0),
            (Object::Real(-90.0), -1.0),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(sin(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect).abs() < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_exp() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(1), Object::Integer(100), 1.0),
            (Object::Real(1.0), Object::Real(100.0), 1.0),
            (Object::Integer(2), Object::Integer(8), 256.0),
            (Object::Real(2.0), Object::Real(8.0), 256.0),
            (Object::Real(2.0), Object::Real(8.0), 256.0),
            (Object::Real(2.0), Object::Real(8.0), 256.0),
            (Object::Integer(8), Object::Real(1.0 / 3.0), 2.0),
            (Object::Real(8.0), Object::Real(1.0 / 3.0), 2.0),
            (Object::Integer(9), Object::Real(0.5), 3.0),
            (Object::Real(9.0), Object::Real(0.5), 3.0),
            (Object::Integer(-9), Object::Integer(-1), -0.111111),
            (Object::Real(-9.0), Object::Real(-1.0), -0.111111),
        ];

        for (base, exponent, expect) in cases {
            execution_state.operand_stack.push(base);
            execution_state.operand_stack.push(exponent);

            assert!(exp(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect) < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_ln() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(10), 2.302585),
            (Object::Real(10.0), 2.302585),
            (Object::Integer(100), 4.60517),
            (Object::Real(100.0), 4.60517),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(ln(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect) < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_log() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        let cases = [
            (Object::Integer(10), 1.0),
            (Object::Real(10.0), 1.0),
            (Object::Integer(100), 2.0),
            (Object::Real(100.0), 2.0),
        ];

        for (input, expect) in cases {
            execution_state.operand_stack.push(input);

            assert!(log(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Real(value)) = execution_state.operand_stack.pop() else {
                return Err("expected real object".into());
            };
            assert!((value - expect) < ERROR_MARGIN);
        }

        Ok(())
    }

    #[test]
    fn test_rand() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        for _ in 0..5 {
            assert!(rand(&mut execution_state).is_ok());
            assert_eq!(1, execution_state.operand_stack.count());
            let Some(Object::Integer(value)) = execution_state.operand_stack.pop() else {
                return Err("expected integer object".into());
            };
            assert!(value >= 0);
        }

        Ok(())
    }

    #[test]
    fn test_srand() -> Result<(), Box<dyn error::Error>> {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(100));
        assert!(srand(&mut execution_state).is_ok());
        assert!(rand(&mut execution_state).is_ok());
        let Some(Object::Integer(initial_value)) = execution_state.operand_stack.pop() else {
            return Err("expected integer object".into());
        };

        execution_state.operand_stack.push(Object::Integer(1));
        assert!(srand(&mut execution_state).is_ok());
        assert!(rand(&mut execution_state).is_ok());
        let Some(Object::Integer(intmd_value)) = execution_state.operand_stack.pop() else {
            return Err("expected integer object".into());
        };
        assert_ne!(initial_value, intmd_value);

        execution_state.operand_stack.push(Object::Integer(100));
        assert!(srand(&mut execution_state).is_ok());
        assert!(rand(&mut execution_state).is_ok());
        let Some(Object::Integer(end_value)) = execution_state.operand_stack.pop() else {
            return Err("expected integer object".into());
        };
        assert_eq!(initial_value, end_value);

        Ok(())
    }

    #[test]
    fn test_rrand() {
        let mut execution_state = ExecutionState::new();

        execution_state.operand_stack.push(Object::Integer(1));
        assert!(srand(&mut execution_state).is_ok());
        for _ in 0..3 {
            assert!(rand(&mut execution_state).is_ok());
        }
        assert!(rrand(&mut execution_state).is_ok());
        assert_eq!(
            Some(Object::Integer(1)),
            execution_state.operand_stack.pop(),
        );
        execution_state.operand_stack.clear();
    }
}
