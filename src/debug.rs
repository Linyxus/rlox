use crate::chunk::{Inst, Chunk};
use crate::value::Value;
use crate::obj::Obj;

pub fn display_inst(inst: &Inst, chunk: &Chunk) {
    match inst {
        Inst::RETURN => println!("RETURN"),
        Inst::CONSTANT { idx } => {
            let constant = &chunk.value_array.data[*idx];
            println!("CONSTANT {} ({})", idx, show_value(constant));
        },
        Inst::OP_NEGATE => println!("OP_NEGATE"),
        Inst::OP_ADD => println!("OP_ADD"),
        Inst::OP_SUB => println!("OP_SUB"),
        Inst::OP_DIV => println!("OP_DIV"),
        Inst::OP_MUL => println!("OP_MUL"),
        Inst::OP_NOT => println!("OP_NOT"),
        Inst::OP_EQ => println!("OP_EQ"),
        Inst::OP_GT => println!("OP_GT"),
        Inst::OP_LT => println!("OP_LT"),
        Inst::OP_KCALL { tp } => println!("OP_KCALL {}", tp.clone() as u32),
        Inst::OP_POP => println!("OP_POP"),
        Inst::OP_DEFINE_GLOBAL { name_idx } => {
            let var_name = &chunk.value_array.data[*name_idx];
            println!("DEFINE_GLOBAL {} ({})", name_idx, show_value(var_name));
        },
        Inst::OP_GET_GLOBAL { name_idx } => {
            let var_name = &chunk.value_array.data[*name_idx];
            println!("GET_GLOBAL {} ({})", name_idx, show_value(var_name));
        },
    }
}

pub fn show_value(value: &Value) -> String {
    match value {
        Value::DOUBLE { data } => format!("{}", data),
        Value::BOOL { data } => format!("{}", data),
        Value::NIL => "nil".into(),
        Value::OBJ { data } => show_obj(data),
        Value::EMPTY => "EMPTY".to_string()
    }
}

pub fn show_obj(obj: &Obj) -> String {
    match obj {
        Obj::Str { data } => format!("'{}'", data),
    }
}

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("===== {} =====", name);
    for idx in 0..chunk.data.len() {
        let inst = &chunk.data[idx];
        let lineno = &chunk.lines[idx];
        if idx == 0 || *lineno != chunk.lines[idx - 1] {
            print!("{:04} ", lineno);
        } else {
            print!("   | ");
        }
        display_inst(inst, chunk);
    }
}


