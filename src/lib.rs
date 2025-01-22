use std::collections;

pub use error::{Error, ErrorKind};
use evaluator::{Defined, Evaluator};
use object::Object;
use scanner::Scanner;
use stack::Stack;
use tokenizer::Tokenizer;

mod encoding;
mod error;
mod evaluator;
mod object;
mod operator;
mod scanner;
mod stack;
mod token;
mod tokenizer;

pub type Result<T> = std::result::Result<T, crate::Error>;

pub fn scan(input: &str) {
    let scanner = Scanner::from(input.chars());
    let tokens = scanner.filter_map(|t| t.ok());
    let objects: Vec<Object> = Tokenizer::from(tokens).filter_map(|obj| obj.ok()).collect();
    println!("{:?}", objects);
}

pub fn eval(input: &str) -> crate::Result<()> {
    let scanner = Scanner::from(input.chars());
    let tokenizer = Tokenizer::from(scanner.filter_map(|t| t.ok()));

    let mut system_dict = collections::HashMap::new();
    load_system_operators(&mut system_dict);

    let mut dicts = Stack::new();
    dicts.push(system_dict);

    let mut evaluator = Evaluator::new(dicts);
    evaluator.evaluate(tokenizer.filter_map(|o| o.ok()))?;

    Ok(())
}

fn load_system_operators(dict: &mut collections::HashMap<String, Defined>) {
    // Stack operators
    dict.insert("dup".to_string(), Defined::Function(operator::dup));
    dict.insert("exch".to_string(), Defined::Function(operator::exch));
    dict.insert("pop".to_string(), Defined::Function(operator::pop));
    dict.insert("copy".to_string(), Defined::Function(operator::copy));
    dict.insert("roll".to_string(), Defined::Function(operator::roll));
    dict.insert("index".to_string(), Defined::Function(operator::index));
    dict.insert("mark".to_string(), Defined::Function(operator::mark));
    dict.insert("clear".to_string(), Defined::Function(operator::clear));
    dict.insert("count".to_string(), Defined::Function(operator::count));
    dict.insert(
        "counttomark".to_string(),
        Defined::Function(operator::counttomark),
    );
    dict.insert(
        "cleartomark".to_string(),
        Defined::Function(operator::cleartomark),
    );

    // Math operators
    dict.insert("add".to_string(), Defined::Function(operator::add));
    dict.insert("div".to_string(), Defined::Function(operator::div));
    dict.insert("idiv".to_string(), Defined::Function(operator::idiv));
    dict.insert("mod".to_string(), Defined::Function(operator::imod));
    dict.insert("mul".to_string(), Defined::Function(operator::mul));
    dict.insert("sub".to_string(), Defined::Function(operator::sub));
    dict.insert("abs".to_string(), Defined::Function(operator::abs));
    dict.insert("neg".to_string(), Defined::Function(operator::neg));
    dict.insert("ceiling".to_string(), Defined::Function(operator::ceiling));
    dict.insert("floor".to_string(), Defined::Function(operator::floor));
    dict.insert("round".to_string(), Defined::Function(operator::round));
    dict.insert(
        "truncate".to_string(),
        Defined::Function(operator::truncate),
    );
    dict.insert("sqrt".to_string(), Defined::Function(operator::sqrt));
    dict.insert("atan".to_string(), Defined::Function(operator::atan));
    dict.insert("cos".to_string(), Defined::Function(operator::cos));
    dict.insert("sin".to_string(), Defined::Function(operator::sin));
}
