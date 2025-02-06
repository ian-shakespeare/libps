use std::{collections::HashMap, f64::consts};

use array_operators::*;
use math_operators::*;
use stack_operators::*;

use crate::{interpreter::InterpreterState, Error, ErrorKind, Object};

mod array_operators;
mod math_operators;
mod stack_operators;

pub fn system_dict() -> HashMap<String, Object> {
    let ops: Vec<(&str, fn(&mut InterpreterState) -> crate::Result<()>)> = vec![
        ("dup", dup),
        ("exch", exch),
        ("pop", |state| {
            state.pop()?;
            Ok(())
        }),
        ("copy", copy),
        ("roll", roll),
        ("index", index),
        ("mark", mark),
        ("clear", clear),
        ("count", count),
        ("counttomark", counttomark),
        ("cleartomark", cleartomark),
        ("add", |state| {
            arithmetic(state, i32::checked_add, |a: f64, b: f64| a + b)
        }),
        ("div", |state| {
            arithmetic(state, |_, _| None, |a: f64, b: f64| a / b)
        }),
        ("idiv", idiv),
        ("imod", imod),
        ("mul", |state| {
            arithmetic(state, i32::checked_mul, |a: f64, b: f64| a * b)
        }),
        ("sub", |state| {
            arithmetic(state, i32::checked_sub, |a: f64, b: f64| a - b)
        }),
        ("abs", |state| num_unary(state, i32::checked_abs, f64::abs)),
        ("neg", |state| {
            num_unary(state, i32::checked_neg, |a: f64| -1.0 * a)
        }),
        ("ceiling", |state| {
            num_unary(state, |a: i32| Some(a), f64::ceil)
        }),
        ("floor", |state| {
            num_unary(state, |a: i32| Some(a), f64::floor)
        }),
        ("round", |state| {
            num_unary(state, |a: i32| Some(a), f64::round)
        }),
        ("truncate", |state| {
            num_unary(state, |a: i32| Some(a), f64::trunc)
        }),
        ("sqrt", |state| real_unary(state, f64::sqrt)),
        ("atan", atan),
        ("cos", cos),
        ("sin", sin),
        ("exp", |state| {
            arithmetic(state, |_, _| None, |base: f64, exp: f64| base.powf(exp))
        }),
        ("ln", |state| real_unary(state, f64::ln)),
        ("log", |state| real_unary(state, f64::log10)),
        ("rand", rand),
        ("srand", srand),
        ("rrand", rrand),
        ("array", array),
        ("[", mark),
        ("]", endarray),
        ("length", length),
        ("get", get),
        ("put", put),
        ("getinterval", getinterval),
        ("putinterval", putinterval),
        ("astore", astore),
        ("aload", aload),
        ("packedarray", packedarray),
        ("setpacking", setpacking),
        ("currentpacking", currentpacking),
    ];

    let mut system_dict = HashMap::new();
    for (key, op) in ops {
        system_dict.insert(key.to_string(), Object::Operator(op));
    }

    system_dict
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

fn is_valid_real(n: f64) -> bool {
    n.is_finite() && !n.is_nan()
}
