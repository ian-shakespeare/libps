use std::collections;

use crate::{object::Object, stack::Stack};

#[allow(dead_code)]
pub struct Evaluator {
    operand_stack: Stack<Object>,
    dict_stack: Stack<collections::HashMap<String, Object>>,
    execution_stack: Stack<Object>,
}
