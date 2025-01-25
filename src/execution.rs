use std::collections;

use crate::{object::Object, operators, rand::RandomNumberGenerator, stack::Stack};

#[allow(dead_code)]
pub enum Procedure {
    Defined(Object),
    System(fn(&mut ExecutionState) -> crate::Result<()>),
}

#[allow(dead_code)]
pub struct ExecutionState {
    pub operand_stack: Stack<Object>,
    pub execution_stack: Stack<Object>,
    pub dict_stack: Stack<collections::HashMap<String, Procedure>>,
    pub rng: RandomNumberGenerator,
}

impl Default for ExecutionState {
    fn default() -> Self {
        let mut system_dict = collections::HashMap::new();

        // Stack operators
        system_dict.insert("dup".to_string(), Procedure::System(operators::dup));
        system_dict.insert("exch".to_string(), Procedure::System(operators::exch));
        system_dict.insert("pop".to_string(), Procedure::System(operators::pop));
        system_dict.insert("copy".to_string(), Procedure::System(operators::copy));
        system_dict.insert("roll".to_string(), Procedure::System(operators::roll));
        system_dict.insert("index".to_string(), Procedure::System(operators::index));
        system_dict.insert("mark".to_string(), Procedure::System(operators::mark));
        system_dict.insert("clear".to_string(), Procedure::System(operators::clear));
        system_dict.insert("count".to_string(), Procedure::System(operators::count));
        system_dict.insert(
            "counttomark".to_string(),
            Procedure::System(operators::counttomark),
        );
        system_dict.insert(
            "cleartomark".to_string(),
            Procedure::System(operators::cleartomark),
        );

        // Math operators
        system_dict.insert("add".to_string(), Procedure::System(operators::add));
        system_dict.insert("div".to_string(), Procedure::System(operators::div));
        system_dict.insert("idiv".to_string(), Procedure::System(operators::idiv));
        system_dict.insert("mod".to_string(), Procedure::System(operators::imod));
        system_dict.insert("mul".to_string(), Procedure::System(operators::mul));
        system_dict.insert("sub".to_string(), Procedure::System(operators::sub));
        system_dict.insert("abs".to_string(), Procedure::System(operators::abs));
        system_dict.insert("neg".to_string(), Procedure::System(operators::neg));
        system_dict.insert("ceiling".to_string(), Procedure::System(operators::ceiling));
        system_dict.insert("floor".to_string(), Procedure::System(operators::floor));
        system_dict.insert("round".to_string(), Procedure::System(operators::round));
        system_dict.insert(
            "truncate".to_string(),
            Procedure::System(operators::truncate),
        );
        system_dict.insert("sqrt".to_string(), Procedure::System(operators::sqrt));
        system_dict.insert("atan".to_string(), Procedure::System(operators::atan));
        system_dict.insert("cos".to_string(), Procedure::System(operators::cos));
        system_dict.insert("sin".to_string(), Procedure::System(operators::sin));
        system_dict.insert("exp".to_string(), Procedure::System(operators::exp));
        system_dict.insert("ln".to_string(), Procedure::System(operators::ln));
        system_dict.insert("log".to_string(), Procedure::System(operators::log));
        system_dict.insert("rand".to_string(), Procedure::System(operators::rand));
        system_dict.insert("srand".to_string(), Procedure::System(operators::srand));
        system_dict.insert("rrand".to_string(), Procedure::System(operators::rrand));

        // Composite operators
        system_dict.insert("array".to_string(), Procedure::System(operators::array));
        system_dict.insert("[".to_string(), Procedure::System(operators::startarray));
        system_dict.insert("]".to_string(), Procedure::System(operators::endarray));
        system_dict.insert("length".to_string(), Procedure::System(operators::length));
        system_dict.insert("get".to_string(), Procedure::System(operators::get));
        system_dict.insert("put".to_string(), Procedure::System(operators::put));
        system_dict.insert(
            "getinterval".to_string(),
            Procedure::System(operators::getinterval),
        );
        system_dict.insert(
            "putinterval".to_string(),
            Procedure::System(operators::putinterval),
        );
        system_dict.insert("astore".to_string(), Procedure::System(operators::astore));
        system_dict.insert("aload".to_string(), Procedure::System(operators::aload));

        let mut dict_stack = Stack::new();
        dict_stack.push(system_dict);

        Self {
            operand_stack: Stack::new(),
            execution_stack: Stack::new(),
            rng: RandomNumberGenerator::default(),
            dict_stack,
        }
    }
}

#[allow(dead_code)]
impl ExecutionState {
    pub fn new() -> Self {
        Self {
            operand_stack: Stack::new(),
            execution_stack: Stack::new(),
            dict_stack: Stack::new(),
            rng: RandomNumberGenerator::default(),
        }
    }
}
