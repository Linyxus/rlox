use std::any::{Any, TypeId};
use crate::obj::Obj;
use std::rc::Rc;

#[derive(Debug)]
#[derive(Clone)]
pub enum Value {
    DOUBLE { data: f64 },
    BOOL { data: bool },
    NIL,
    OBJ { data: Rc<Obj> },
    EMPTY,
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::DOUBLE { data: _ } => "double",
            Value::BOOL { data: _ } => "bool",
            Value::NIL => "nil",
            Value::OBJ { data: _ } => "obj",
            _ => { panic!("Retrieving typename on empty value") }
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::OBJ { data } => match data.as_ref() {
                Obj::Str { data: _ } => true
            },
            _ => false
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::OBJ { data } => match data.as_ref() {
                Obj::Str { data: s } => Option::Some(s.as_ref())
            },
            _ => Option::None
        }
    }

    pub fn create_string_obj(s: String) -> Value {
        let obj = Obj::Str { data: s };
        Value::OBJ { data: Rc::new(obj) }
    }
}

#[derive(Debug)]
pub struct ValueArray {
    pub data: Vec<Value>,
}

impl ValueArray {
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.data.push(value);
        self.data.len() - 1
    }

    pub fn read(&self, idx: usize) -> Value {
        self.data[idx].clone()
    }

    pub fn new() -> ValueArray {
        ValueArray {
            data: Vec::new()
        }
    }
}

