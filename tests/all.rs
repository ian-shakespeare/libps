use std::{error, fs};

use libps::{evaluate, Context};

type TestResult = Result<(), Box<dyn error::Error>>;

fn run_test(test_name: &str) -> TestResult {
    let input = fs::read_to_string(format!("tests/{test_name}"))?;
    let mut ctx = Context::with_debug_utils();
    evaluate(&mut ctx, &input)?;

    Ok(())
}

#[test]
fn test_pop() -> TestResult {
    run_test("test_pop.ps")
}

#[test]
fn test_exch() -> TestResult {
    run_test("test_exch.ps")
}

#[test]
fn test_dup() -> TestResult {
    run_test("test_dup.ps")
}

#[test]
fn test_copy() -> TestResult {
    run_test("test_copy.ps")
}

#[test]
fn test_index() -> TestResult {
    run_test("test_index.ps")
}

#[test]
fn test_roll() -> TestResult {
    run_test("test_roll.ps")
}

#[test]
fn test_clear() -> TestResult {
    run_test("test_clear.ps")
}

#[test]
fn test_count() -> TestResult {
    run_test("test_count.ps")
}

#[test]
fn test_mark() -> TestResult {
    run_test("test_mark.ps")
}

#[test]
fn test_cleartomark() -> TestResult {
    run_test("test_cleartomark.ps")
}

#[test]
fn test_countotmark() -> TestResult {
    run_test("test_counttomark.ps")
}

#[test]
fn test_add() -> TestResult {
    run_test("test_add.ps")
}

#[test]
fn test_div() -> TestResult {
    run_test("test_div.ps")
}

#[test]
fn test_idiv() -> TestResult {
    run_test("test_idiv.ps")
}

#[test]
fn test_array() -> TestResult {
    run_test("test_array.ps")
}

#[test]
fn test_length() -> TestResult {
    run_test("test_length.ps")
}

#[test]
fn test_get() -> TestResult {
    run_test("test_get.ps")
}

#[test]
fn test_put() -> TestResult {
    run_test("test_put.ps")
}

#[test]
fn test_getinterval() -> TestResult {
    run_test("test_getinterval.ps")
}

#[test]
fn test_forall() -> TestResult {
    run_test("test_forall.ps")
}

#[test]
fn test_dict() -> TestResult {
    run_test("test_dict.ps")
}
