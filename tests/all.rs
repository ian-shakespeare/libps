use std::{error, fs};

use libps::{evaluate, Context};

mod common;

type TestResult = Result<(), Box<dyn error::Error>>;

fn run_test(test_name: &str) -> TestResult {
    let input = fs::read_to_string(format!("tests/{test_name}"))?;
    let mut ctx = Context::default();

    let assert_definitions = include_str!("assert.ps");
    evaluate(&mut ctx, assert_definitions).expect("failed to evaluate assert definitions");

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
fn test_mod() -> TestResult {
    run_test("test_mod.ps")
}

#[test]
fn test_sub() -> TestResult {
    run_test("test_sub.ps")
}

#[test]
fn test_abs() -> TestResult {
    run_test("test_abs.ps")
}

#[test]
fn test_neg() -> TestResult {
    run_test("test_neg.ps")
}

#[test]
fn test_ceiling() -> TestResult {
    run_test("test_ceiling.ps")
}

#[test]
fn test_floor() -> TestResult {
    run_test("test_floor.ps")
}

#[test]
fn test_round() -> TestResult {
    run_test("test_round.ps")
}

#[test]
fn test_truncate() -> TestResult {
    run_test("test_truncate.ps")
}

#[test]
fn test_sqrt() -> TestResult {
    run_test("test_sqrt.ps")
}

#[test]
fn test_atan() -> TestResult {
    run_test("test_atan.ps")
}

#[test]
fn test_cos() -> TestResult {
    run_test("test_cos.ps")
}

#[test]
fn test_sin() -> TestResult {
    run_test("test_sin.ps")
}

#[test]
fn test_exp() -> TestResult {
    run_test("test_exp.ps")
}

#[test]
fn test_ln() -> TestResult {
    run_test("test_ln.ps")
}

#[test]
fn test_log() -> TestResult {
    run_test("test_log.ps")
}

#[test]
fn test_rand() -> TestResult {
    run_test("test_rand.ps")
}

// Covers both `srand` & `rrand`
#[test]
fn test_srand() -> TestResult {
    run_test("test_srand.ps")
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
fn test_putinterval() -> TestResult {
    run_test("test_putinterval.ps")
}

#[test]
fn test_astore() -> TestResult {
    run_test("test_astore.ps")
}

#[test]
fn test_aload() -> TestResult {
    run_test("test_aload.ps")
}

#[test]
fn test_forall() -> TestResult {
    run_test("test_forall.ps")
}

// Covers both `setpacking` & `currentpacking`
#[test]
fn test_setpacking() -> TestResult {
    run_test("test_setpacking.ps")
}

#[test]
fn test_dict() -> TestResult {
    run_test("test_dict.ps")
}

#[test]
fn test_maxlength() -> TestResult {
    run_test("test_maxlength.ps")
}

#[test]
fn test_begin() -> TestResult {
    run_test("test_begin.ps")
}

#[test]
fn test_end() -> TestResult {
    run_test("test_end.ps")
}

#[test]
fn test_def() -> TestResult {
    run_test("test_def.ps")
}

#[test]
fn test_load() -> TestResult {
    run_test("test_load.ps")
}

#[test]
fn test_store() -> TestResult {
    run_test("test_store.ps")
}
