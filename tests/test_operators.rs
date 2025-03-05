use std::error;

use libps::{evaluate, Context};

type TestResult = Result<(), Box<dyn error::Error>>;

#[test]
fn test_dup() -> TestResult {
    let input = include_str!("test_dup.ps");
    let mut ctx = Context::with_test_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}

#[test]
fn test_forall() -> TestResult {
    let input = include_str!("test_forall.ps");
    let mut ctx = Context::with_test_utils();
    evaluate(&mut ctx, input)?;

    Ok(())
}
