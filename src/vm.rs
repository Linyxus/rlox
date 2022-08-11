use crate::chunk::{ Chunk, Inst, KMethod };
use crate::value::Value;
use crate::debug::{show_value, display_inst};

use std::collections::HashMap;

use phf::phf_map;

const STACK_MAX: usize = 128;

#[derive(Debug)]
pub struct VM {
    pub chunk: Chunk,
    pc: u32,

    stack: Vec<Value>,
    sp: u32,

    globals: HashMap<String, Value>,

    enable_trace: bool,
}

#[derive(Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

type KernalOp = fn(&mut VM);

fn kop_print(vm: &mut VM) {
    let v = vm.pop().expect("Expecting non-empty stack");
    println!("{}", show_value(v));
}

static KERNAL_METHODS: phf::Map<u8, KernalOp> = phf_map! {
    0u8 => kop_print,
};

type UnOp = fn(&Value) -> Value;

type BinOp = fn(&Value, &Value) -> Value;

fn op_negate(v: &Value) -> Value {
    match v {
        Value::DOUBLE { data } => Value::DOUBLE { data: -data },
        _ => Value::EMPTY
    }
}

fn op_not(v: &Value) -> Value {
    match v {
        Value::BOOL { data } => Value::BOOL { data: !data },
        Value::NIL => Value::BOOL { data: true },
        _ => Value::EMPTY
    }
}

fn op_add(v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::DOUBLE { data: x1 }, Value::DOUBLE { data: x2 }) => Value::DOUBLE { data: x1 + x2 },
        (Value::OBJ { data: obj1 }, Value::OBJ { data: obj2 }) => {
            if v1.is_string() && v2.is_string() {
                let s1 = v1.as_string().expect("Failed to convert value to string.");
                let s2 = v2.as_string().expect("Failed to convert value to string.");
                let mut s = s2.to_string();
                s.push_str(s1);
                Value::create_string_obj(s)
            } else {
                panic!("Could not add these two values: {:?} and {:?}", v1, v2)
            }
        }
        _ => Value::EMPTY
    }
}

fn op_sub(v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::DOUBLE { data: x1 }, Value::DOUBLE { data: x2 }) => Value::DOUBLE { data: x2 - x1 },
        _ => Value::EMPTY
    }
}

fn op_mul(v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::DOUBLE { data: x1 }, Value::DOUBLE { data: x2 }) => Value::DOUBLE { data: x1 * x2 },
        _ => Value::EMPTY
    }
}

fn op_div(v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::DOUBLE { data: x1 }, Value::DOUBLE { data: x2 }) => Value::DOUBLE { data: x2 / x1 },
        _ => Value::EMPTY
    }
}

fn op_eq(v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::DOUBLE { data: x1 }, Value::DOUBLE { data: x2 }) => Value::BOOL { data: x1 == x2 },
        (Value::BOOL { data: x1 }, Value::BOOL { data: x2 }) => Value::BOOL { data: x1 == x2 },
        (Value::NIL, Value::NIL) => Value::BOOL { data: true },
        (v1, v2) if v1.is_string() && v2.is_string() => {
            Value::BOOL { data: v1.as_string().expect("") == v2.as_string().expect("") }
        },
        _ => Value::BOOL { data: false },
    }
}

fn op_gt(v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::DOUBLE { data: x1 }, Value::DOUBLE { data: x2 }) => Value::BOOL { data: x2 > x1 },
        _ => Value::EMPTY
    }
}

fn op_lt(v1: &Value, v2: &Value) -> Value {
    match (v1, v2) {
        (Value::DOUBLE { data: x1 }, Value::DOUBLE { data: x2 }) => Value::BOOL { data: x2 < x1 },
        _ => Value::EMPTY
    }
}


macro_rules! both_matches {
    ($e1:expr, $e2: expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        matches!($e1, $( $pattern )|+ $( if $guard )?) && matches!($e2, $( $pattern )|+ $( if $guard )?)
    }
}

impl VM {
    pub fn new(chunk: Chunk) -> VM {
        VM { chunk, pc: 0, stack: VM::create_empty_stack(), sp: 0, globals: HashMap::new(), enable_trace: false }
    }

    fn create_empty_stack() -> Vec<Value> {
        let mut res: Vec<Value> = Vec::new();
        for i in 0..STACK_MAX {
            res.push(Value::EMPTY);
        }
        res
    }

    pub fn runtime_error(&self, msg: String) {
        let lineno = self.chunk.lines[self.pc as usize];
        println!("{}", msg);
        eprintln!("[line {}] in script", lineno);
    }

    pub fn trace_on(&mut self) {
        self.enable_trace = true;
    }

    pub fn trace_off(&mut self) {
        self.enable_trace = false;
    }

    pub fn update_global(&mut self, name: String, v: Value) {
        self.globals.insert(name, v);
    }

    fn unop_typecheck(&mut self, checker: fn(&Value) -> bool, desc: &str) -> bool {
        let v = self.peek();
        let res = checker(&v);

        if !res {
            self.runtime_error(format!("Expecting operand of type {}", desc));
        }

        res
    }

    fn binop_typecheck(&mut self, checker: fn(&Value) -> bool, desc: &str) -> bool {
        let v1 = self.peek_at(0);
        let v2 = self.peek_at(1);

        let res = checker(&v1) && checker(&v2);

        if !res {
            self.runtime_error(format!("Expecting operands of type {}", desc));
        }

        res
    }

    fn binop_typecheck_both(&mut self, checker: fn(&Value, &Value) -> bool, desc: &str) -> bool {
        let v1 = self.peek_at(0);
        let v2 = self.peek_at(1);

        let res = checker(&v1, &v2);

        if !res {
            self.runtime_error(format!("Expecting operands of type {}", desc));
        }

        res
    }

    fn lift_unop(&mut self, op: UnOp) {
        match self.pop() {
            Option::None => self.push(Value::EMPTY),
            Option::Some(v0) => {
                let v1 = op(&v0);
                self.push(v1)
            }
        }
    }

    fn lift_binop(&mut self, op: BinOp) {
        let v1;
        let v2;

        match self.pop() {
            Option::None => { self.push(Value::EMPTY); return; },
            Option::Some(v) => {
                v1 = v.clone();
            }
        }

        match self.pop() {
            Option::None => { self.push(Value::EMPTY); return; },
            Option::Some(v) => {
                v2 = v.clone();
            }
        }

        self.push(op(&v1, &v2))
    }

    fn define_variable(&mut self, name_idx: usize) {
        let v = self.chunk.value_array.read(name_idx);
        let varname = v.as_string().expect("Expecting string as variable name");
        let v = self.pop().expect("Expecting non-empty stack").clone();
        self.update_global(varname.into(), v);
    }

    pub fn run(&mut self) -> InterpretResult {
        let res = loop {

            if self.enable_trace {
                self.display_stack();
                self.display_globals();
                display_inst(&self.chunk.data[self.pc as usize], &self.chunk)
            }

            let inst = self.fetch();

            match inst {
                Inst::RETURN => {
                    break InterpretResult::Ok
                },
                Inst::OP_POP => {
                    self.pop();
                },
                Inst::OP_KCALL { tp } => {
                    let kop = KERNAL_METHODS.get(&(tp.clone() as u8)).expect("Unsupported kernal method");
                    kop(self);
                },
                Inst::OP_DEFINE_GLOBAL { name_idx } => {
                    self.define_variable(name_idx.clone());
                },
                Inst::CONSTANT { idx } => {
                    let val = self.chunk.value_array.read(*idx);
                    self.push(val);
                },
                Inst::OP_NEGATE => {
                    self.unop_typecheck(|v| matches!(v, Value::DOUBLE { data: _ }), "number");
                    self.lift_unop(op_negate)
                },
                Inst::OP_NOT => {
                    self.unop_typecheck(|v| matches!(v, Value::BOOL { data: _ }) || matches!(v, Value::NIL), "boolean or nil");
                    self.lift_unop(op_not)
                }
                Inst::OP_ADD => {
                    self.binop_typecheck_both(
                        |v1, v2| both_matches!(v1, v2, Value::DOUBLE { data: _ }) || (v1.is_string() && v2.is_string()),
                        "number or string"
                    );
                    self.lift_binop(op_add)
                },
                Inst::OP_SUB => {
                    self.binop_typecheck(|v| matches!(v, Value::DOUBLE { data: _ }), "number");
                    self.lift_binop(op_sub)
                },
                Inst::OP_DIV => {
                    self.binop_typecheck(|v| matches!(v, Value::DOUBLE { data: _ }), "number");
                    self.lift_binop(op_div)
                },
                Inst::OP_MUL => {
                    self.binop_typecheck(|v| matches!(v, Value::DOUBLE { data: _ }), "number");
                    self.lift_binop(op_mul)
                },
                Inst::OP_EQ => {
                    self.lift_binop(op_eq)
                },
                Inst::OP_GT => {
                    self.binop_typecheck(|v| matches!(v, Value::DOUBLE { data: _ }), "number");
                    self.lift_binop(op_gt)
                },
                Inst::OP_LT => {
                    self.binop_typecheck(|v| matches!(v, Value::DOUBLE { data: _ }), "number");
                    self.lift_binop(op_lt)
                },
                _ => break InterpretResult::RuntimeError
            }

            self.step();
        };
        res
    }

    pub fn fetch(&self) -> &Inst {
        &self.chunk.data[self.pc as usize]
    }

    pub fn step(&mut self) {
        self.pc = self.pc + 1;
    }

    pub fn push(&mut self, value: Value) {
        let idx = self.sp as usize;
        self.stack[idx] = value;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> Option<&Value> {
        if self.sp <= 0 {
            None
        } else {
            self.sp -= 1;
            Some(&self.stack[self.sp as usize])
        }
    }

    pub fn peek_opt(&self) -> Option<&Value> {
        if self.sp <= 0 {
            None
        } else {
            Some(&self.stack[self.sp as usize - 1])
        }
    }

    pub fn peek_at_opt(&self, idx: u32) -> Option<&Value> {
        let i = self.sp - idx;

        if i <= 0 {
            None
        } else {
            Some(&self.stack[i as usize - 1])
        }
    }

    pub fn peek(&self) -> &Value {
        self.peek_opt().expect("Peeking value, but found an empty stack")
    }

    pub fn peek_at(&self, i: u32) -> &Value {
        self.peek_at_opt(i).expect("Peeking value, but found an empty stack")
    }

    pub fn display_stack(&self) {
        print!(" STACK: ");
        for i in 0..self.sp {
            let value = &self.stack[i as usize];
            print!("[ {} ] ", show_value(value));
        }
        println!("");
    }

    pub fn display_globals(&self) {
        print!(" GLOBALS: ");
        for (k, v) in &self.globals {
            print!("{} => {}; ", k, show_value(v));
        }
        println!("");
    }
}

