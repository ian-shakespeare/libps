use std::f64::consts;

pub use array_operators::*;
pub use debug_operators::*;
pub use dict_operators::*;
pub use error_operators::*;
pub use math_operators::*;
pub use misc_operators::*;
pub use relational_operators::*;
pub use stack_operators::*;
pub use type_operators::*;

use crate::{Error, ErrorKind};

mod array_operators;
mod debug_operators;
mod dict_operators;
mod error_operators;
mod math_operators;
mod misc_operators;
mod relational_operators;
mod stack_operators;
mod type_operators;

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

fn i32_to_usize(i: i32) -> crate::Result<usize> {
    let u: usize = match i.try_into() {
        Ok(i) => Ok(i),
        Err(_) => Err(Error::from(ErrorKind::LimitCheck)),
    }?;

    Ok(u)
}

fn is_valid_real(n: f64) -> bool {
    n.is_finite() && !n.is_nan()
}
