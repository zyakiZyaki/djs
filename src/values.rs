use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

// ============================================================================
// PROMISES
// ============================================================================

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PromiseStatus { Pending, Fulfilled, Rejected }

#[derive(Clone, Debug)]
pub struct PromiseState {
    pub status: PromiseStatus,
    pub value: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct PromiseJob {
    pub on_fulfilled: Option<Value>,
    pub on_rejected: Option<Value>,
    pub on_finally: Option<Value>,
    pub target: Rc<RefCell<PromiseState>>,
    pub result: Rc<RefCell<PromiseState>>,
}

impl PromiseState {
    pub fn new() -> Self { Self { status: PromiseStatus::Pending, value: None } }
}

// ============================================================================
// VALUES
// ============================================================================

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Closure(Rc<Closure>),
    Promise(Rc<RefCell<PromiseState>>),
    Resolver(Rc<RefCell<PromiseState>>),
    Rejector(Rc<RefCell<PromiseState>>),
    BytecodeClosure(crate::vm::vm::BytecodeClosure),
    Builtin(String),  // Name of builtin function
    Expect { value: Box<Value>, negate: bool },  // For expect() chaining
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(_), Value::Object(_)) => true,
            (Value::Closure(_), Value::Closure(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub params: Vec<crate::parser::Param>,
    pub body: Vec<crate::parser::Stmt>,
    pub env: Environment,
    pub allowed: std::collections::HashSet<String>,
}

pub type Environment = Rc<RefCell<Env>>;

#[derive(Debug)]
pub struct Env {
    bindings: HashMap<String, Value>,
    pub parent: Option<Environment>,
}

impl Env {
    pub fn new(parent: Option<Environment>) -> Self {
        Env { bindings: HashMap::new(), parent }
    }

    pub fn get(name: &str, env: &Environment) -> Option<Value> {
        if let Some(val) = env.borrow().bindings.get(name) {
            Some(val.clone())
        } else if let Some(parent) = &env.borrow().parent {
            Self::get(name, parent)
        } else {
            None
        }
    }

    pub fn get_local(name: &str, env: &Environment) -> Option<Value> {
        env.borrow().bindings.get(name).cloned()
    }

    pub fn set(name: &str, value: Value, env: &Environment) {
        env.borrow_mut().bindings.insert(name.to_string(), value);
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

pub fn format_value(val: &Value) -> String {
    match val {
        Value::Number(n) => {
            if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{}", n) }
        }
        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::Str(s) => s.clone(),
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Object(_) => "[object]".to_string(),
        Value::Closure(_) => "[function]".to_string(),
        Value::BytecodeClosure(_) => "[function]".to_string(),
        Value::Promise(_) | Value::Resolver(_) | Value::Rejector(_) => "[promise]".to_string(),
        Value::Builtin(name) => format!("[builtin: {}]", name),
        Value::Expect { value, negate: _ } => format!("[expect: {}]", format_value(value)),
    }
}

pub fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Number(n) => *n != 0.0,
        Value::Bool(b) => *b,
        Value::Str(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(_) => true,
        Value::Closure(_) | Value::BytecodeClosure(_) | Value::Promise(_) | Value::Resolver(_) | Value::Rejector(_) => true,
        Value::Builtin(_) => true,
        Value::Expect { value, negate: _ } => is_truthy(value),
    }
}

pub fn is_falsey(val: &Value) -> bool { !is_truthy(val) }

use crate::parser::Param;

pub fn param_name(p: &Param) -> &str {
    match p {
        Param::Name(n) => n,
        Param::Destructure(_) => "_",
        Param::Rest(_) => "_",
    }
}

pub fn bind_param(param: &Param, value: Value, env: &Environment) -> Result<(), String> {
    match param {
        Param::Name(n) => {
            Env::set(n, value, env);
            Ok(())
        }
        Param::Destructure(fields) => {
            if let Value::Object(map) = &value {
                for field in fields {
                    if let Some(val) = map.get(field) {
                        Env::set(field, val.clone(), env);
                    } else {
                        return Err(format!("Destructuring: field '{}' not found in object", field));
                    }
                }
                Ok(())
            } else {
                Err(format!("Cannot destructure non-object value: {:?}", value))
            }
        }
        Param::Rest(_) => {
            Ok(())
        }
    }
}
