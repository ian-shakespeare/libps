use std::error;
/*
use libps::{ErrorKind, Evaluator, Object, Scanner};

#[test]
fn test_dup_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let input = "1 dup";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    let mut operand_stack = evaluator.dump_operand_stack();
    assert_eq!(2, operand_stack.count());
    assert_eq!(Some(Object::Integer(1)), operand_stack.pop());
    assert_eq!(Some(Object::Integer(1)), operand_stack.pop());

    Ok(())
}

#[test]
fn test_exch_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let input = "1 2 exch";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    let mut operand_stack = evaluator.dump_operand_stack();
    assert_eq!(Some(Object::Integer(1)), operand_stack.pop());
    assert_eq!(Some(Object::Integer(2)), operand_stack.pop());

    let input = "1 exch";
    let scanner = Scanner::from(input.chars());
    assert!(evaluator
        .evaluate(scanner.filter_map(|o| o.ok()))
        .is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

    let input = "exch";
    let scanner = Scanner::from(input.chars());
    assert!(evaluator
        .evaluate(scanner.filter_map(|o| o.ok()))
        .is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

    Ok(())
}

#[test]
fn test_pop_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let input = "1 2 pop";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    let mut operand_stack = evaluator.dump_operand_stack();
    assert_eq!(1, operand_stack.count());
    assert_eq!(Some(Object::Integer(1)), operand_stack.pop());

    let input = "pop";
    let scanner = Scanner::from(input.chars());
    assert!(evaluator
        .evaluate(scanner.filter_map(|o| o.ok()))
        .is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

    Ok(())
}

#[test]
fn test_copy_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let input = "1 2 3 2 copy";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    // Stack should be: 3 2 3 2 1 |
    let mut operand_stack = evaluator.dump_operand_stack();
    assert_eq!(5, operand_stack.count());
    assert_eq!(Some(Object::Integer(3)), operand_stack.pop());
    assert_eq!(Some(Object::Integer(2)), operand_stack.pop());
    assert_eq!(Some(Object::Integer(3)), operand_stack.pop());
    assert_eq!(Some(Object::Integer(2)), operand_stack.pop());
    assert_eq!(Some(Object::Integer(1)), operand_stack.pop());

    let input = "copy";
    let scanner = Scanner::from(input.chars());
    assert!(evaluator
        .evaluate(scanner.filter_map(|o| o.ok()))
        .is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

    let input = "[ 1 2 3 ] [ 4 5 6 ] copy";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    // Stack should be: [ 1 2 3 ] |
    let mut operand_stack = evaluator.dump_operand_stack();
    assert_eq!(1, operand_stack.count());
    let Some(Object::Array(arr)) = operand_stack.pop() else {
        return Err("expected array object".into());
    };
    let arr = arr.borrow();
    assert_eq!(Some(Object::Integer(1)), arr.get(0).cloned());
    assert_eq!(Some(Object::Integer(2)), arr.get(1).cloned());
    assert_eq!(Some(Object::Integer(3)), arr.get(2).cloned());

    let input = "[ 1 2 3 ] [ 4 5 ] copy";
    let scanner = Scanner::from(input.chars());
    assert!(evaluator
        .evaluate(scanner.filter_map(|o| o.ok()))
        .is_err_and(|e| e.kind() == ErrorKind::RangeCheck));

    let input = "1 [ 1 2 3 ] copy";
    let scanner = Scanner::from(input.chars());
    assert!(evaluator
        .evaluate(scanner.filter_map(|o| o.ok()))
        .is_err_and(|e| e.kind() == ErrorKind::TypeCheck));

    let input = "[ 1 2 3 ] copy";
    let scanner = Scanner::from(input.chars());
    assert!(evaluator
        .evaluate(scanner.filter_map(|o| o.ok()))
        .is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

    Ok(())
}

#[test]
fn test_roll_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let input = "1 2 3 3 2 roll";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    let mut operand_stack = evaluator.dump_operand_stack();
    assert_eq!(3, operand_stack.count());
    assert_eq!(Some(Object::Integer(1)), operand_stack.pop());
    assert_eq!(Some(Object::Integer(3)), operand_stack.pop());
    assert_eq!(Some(Object::Integer(2)), operand_stack.pop());

    let input = "roll";
    let scanner = Scanner::from(input.chars());
    assert!(evaluator
        .evaluate(scanner.filter_map(|o| o.ok()))
        .is_err_and(|e| e.kind() == ErrorKind::StackUnderflow));

    Ok(())
}

#[test]
fn test_index_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let input = "1 2 3 2 index";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    assert_eq!(
        Some(Object::Integer(1)),
        evaluator.state().operand_stack.top().cloned()
    );

    let input = "pop 1 index";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;
    assert_eq!(
        Some(Object::Integer(2)),
        evaluator.state().operand_stack.top().cloned()
    );

    let input = "pop 0 index";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;
    assert_eq!(
        Some(Object::Integer(3)),
        evaluator.state().operand_stack.top().cloned()
    );

    evaluator.dump_operand_stack();

    let input = "1 2 3 4 index";
    let scanner = Scanner::from(input.chars());
    let e = evaluator.evaluate(scanner.filter_map(|o| o.ok()));
    assert!(e.is_err());
    assert_eq!(ErrorKind::RangeCheck, e.unwrap_err().kind());

    Ok(())
}

#[test]
fn test_mark_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let input = "mark";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    let mut operand_stack = evaluator.dump_operand_stack();
    assert!(matches!(operand_stack.pop(), Some(Object::Mark)));

    Ok(())
}

#[test]
fn test_clear_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let input = "1 2 3 clear";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    let operand_stack = evaluator.dump_operand_stack();
    assert_eq!(0, operand_stack.count());

    Ok(())
}

#[test]
fn test_count_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let cases = [("count", 0), ("1 2 3 count", 3)];

    for (input, expect) in cases {
        let scanner = Scanner::from(input.chars());
        evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

        let mut operand_stack = evaluator.dump_operand_stack();
        assert_eq!(Some(Object::Integer(expect)), operand_stack.pop());
    }

    Ok(())
}

#[test]
fn test_counttomark_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let cases = [("1 mark 2 3 counttomark", 2), ("1 2 3 counttomark", 3)];

    for (input, expect) in cases {
        let scanner = Scanner::from(input.chars());
        evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

        let mut operand_stack = evaluator.dump_operand_stack();
        assert_eq!(Some(Object::Integer(expect)), operand_stack.pop());
    }

    Ok(())
}

#[test]
fn test_cleartomark_operator() -> Result<(), Box<dyn error::Error>> {
    let mut evaluator = Evaluator::default();

    let input = "1 mark 2 3 cleartomark";
    let scanner = Scanner::from(input.chars());
    evaluator.evaluate(scanner.filter_map(|o| o.ok()))?;

    let mut operand_stack = evaluator.dump_operand_stack();
    assert_eq!(1, operand_stack.count());
    assert_eq!(Some(Object::Integer(1)), operand_stack.pop());

    Ok(())
}*/
