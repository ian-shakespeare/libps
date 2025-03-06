use std::error;

use libps::{evaluate, Context};

type TestResult = Result<(), Box<dyn error::Error>>;

#[test]
fn test_dup() -> TestResult {
    let input = include_str!("test_dup.ps");
    let mut ctx = Context::with_debug_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}

#[test]
fn test_array() -> TestResult {
    let input = include_str!("test_array.ps");
    let mut ctx = Context::with_debug_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}

#[test]
fn test_length() -> TestResult {
    let input = include_str!("test_length.ps");
    let mut ctx = Context::with_debug_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}

#[test]
fn test_get() -> TestResult {
    let input = include_str!("test_get.ps");
    let mut ctx = Context::with_debug_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}

#[test]
fn test_put() -> TestResult {
    let input = include_str!("test_put.ps");
    let mut ctx = Context::with_debug_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}

#[test]
fn test_getinterval() -> TestResult {
    let input = include_str!("test_getinterval.ps");
    let mut ctx = Context::with_debug_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}

#[test]
fn test_forall() -> TestResult {
    let input = include_str!("test_forall.ps");
    let mut ctx = Context::with_debug_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}

#[test]
fn test_dict() -> TestResult {
    let input = include_str!("test_dict.ps");
    let mut ctx = Context::with_debug_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}
